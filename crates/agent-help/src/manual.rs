use std::fmt;

use clap::{Arg, ArgAction, Command};

use crate::{Table, Topic, TopicRef, item};

/// A tool's manual, assembled from what is written where it lives: clap's
/// command tree, the topics and the tables. Every answer is Markdown, the
/// text kept as its author wrote it.
///
/// ````
/// use agent_help::{Help, Topic};
/// use clap::{Arg, Command};
///
/// agent_help::binary!("plans");
///
/// /// Run `plans login` once.
/// #[derive(Topic)]
/// #[topic(name = "sign-in")]
/// struct SignIn;
///
/// /// Why `plans` refused a request.
/// #[derive(agent_help::Help)]
/// enum Refusal {
///     /// There are {const:MAX_PLANS} plans already.
///     OverLimit,
/// }
///
/// fn main() {
///     let tool = Command::new("plans")
///         .long_about("Keeps plans.")
///         .subcommand(
///             Command::new("create")
///                 .next_help_heading("Run by the agent")
///                 .long_about("Creates a plan. It may refuse:\n\n{table:refusal}")
///                 .arg(
///                     Arg::new("by")
///                         .long("by")
///                         .default_value("1")
///                         .env("PLANS_BY")
///                         .help("How many plans to create."),
///                 ),
///         );
///     let help = Help::new(tool)
///         .topic::<SignIn>()
///         .table::<Refusal>("refusal")
///         .constant("MAX_PLANS", 10);
///     assert_eq!(
///         help.command_help(&["create"]).unwrap(),
///         "\
/// ## `plans create`
///
/// Creates a plan. It may refuse:
///
/// - `over_limit`: There are 10 plans already.
///
/// ### Usage
///
/// ```
/// plans create [OPTIONS]
/// ```
///
/// ### Arguments
///
/// - `--by <by>`: How many plans to create.
///   - Default: `1`
///   - Environment: `PLANS_BY`
/// "
///     );
///     assert_eq!(
///         help.topic_help("sign-in").unwrap(),
///         "# `plans help sign-in`\n\nRun `plans login` once.\n"
///     );
///     assert!(help.manual().unwrap().starts_with("# `plans`\n\nKeeps plans.\n\n## Usage\n"));
/// }
/// ````
///
/// - A command's text is its `long_about`, else its `about`. With clap's
///   derive, write `#[command(verbatim_doc_comment)]` so its doc comment is
///   kept as written; without it clap rewraps the text.
/// - Every subcommand declares its group, who runs it, where it is defined:
///   `#[command(next_help_heading = "Run by the operator")]`. A nested one
///   is in its parent's group unless it declares its own; a subcommand with
///   none is an error.
/// - Inside any text, `{table:<name>}` is the table registered under that
///   name, as [`Table::markdown`] writes it, and `{const:<NAME>}` the value
///   registered under that name: a number is written once, in code. A table's
///   help may name constants, but not tables. One naming nothing registered
///   is an error.
/// - After its text, a command's answer gives its usage, as clap writes it,
///   then its arguments: positional ones first, then the rest, each named as
///   clap's usage names it, with its help (its `long_help`, else its `help`)
///   and, where set, its default, its environment variable and the values it
///   accepts, each value with its help. A global argument is listed on every
///   command it reaches; a command that takes none has no arguments section.
///   With clap's derive, write `#[arg(verbatim_doc_comment)]`, and
///   `#[value(verbatim_doc_comment)]` on a `ValueEnum`'s variant, so the doc
///   comment is kept as written; without it clap drops a one-line comment's
///   final period.
/// - The manual gives the tool's usage and arguments after its text, and
///   what it says of each command is that command's answer, one heading
///   level down (ADR 0004).
/// - Hidden commands, hidden arguments, hidden values, and clap's own
///   `--help` and `--version` are left out of the manual.
/// - Each answer that finds faults is an [`Error`] naming every one of them.
#[derive(Clone, Debug)]
pub struct Help {
    command: Command,
    topics: Vec<RegisteredTopic>,
    tables: Vec<RegisteredTable>,
    constants: Vec<(&'static str, String)>,
}

/// A topic, as [`Help::topic`] registered it.
#[derive(Clone, Debug)]
struct RegisteredTopic {
    reference: TopicRef,
    markdown: &'static str,
}

/// A table, as [`Help::table`] registered it.
#[derive(Clone, Debug)]
struct RegisteredTable {
    name: &'static str,
    /// Its [`Table::markdown`].
    markdown: fn() -> String,
}

impl Help {
    /// The manual of the tool whose command tree is `command`; its name is
    /// the binary every heading names.
    pub fn new(command: Command) -> Self {
        Self {
            command,
            topics: Vec::new(),
            tables: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Adds topic `T`, after those added before it. It must be shown by this
    /// tool: a topic whose crate named another binary is an error.
    pub fn topic<T: Topic>(mut self) -> Self {
        self.topics.push(RegisteredTopic {
            reference: T::REFERENCE,
            markdown: T::MARKDOWN,
        });
        self
    }

    /// Makes table `E` what `{table:<name>}` stands for.
    pub fn table<E: Table>(mut self, name: &'static str) -> Self {
        self.tables.push(RegisteredTable {
            name,
            markdown: E::markdown,
        });
        self
    }

    /// Makes `value` what `{const:<name>}` stands for.
    pub fn constant(mut self, name: &'static str, value: impl fmt::Display) -> Self {
        self.constants.push((name, value.to_string()));
        self
    }

    /// The whole manual, what `<binary> --help` answers: the tool's own
    /// text, then every command under its group (groups in the order clap's
    /// tree first names them), then every topic.
    pub fn manual(&self) -> Result<String, Error> {
        let mut faults = Faults::default();
        let manual = self.whole(&self.tree(), &mut faults);
        faults.or(manual)
    }

    /// One command's text, what `<binary> <path...> --help` answers; the
    /// empty path is the tool itself, whose answer is the whole manual. A
    /// command that does not exist is an error naming those that do.
    pub fn command_help(&self, path: &[&str]) -> Result<String, Error> {
        if path.is_empty() {
            return self.manual();
        }
        let title = [self.command.get_name()]
            .iter()
            .chain(path)
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
        let tree = self.tree();
        let mut command = &tree;
        for name in path {
            let found = command.get_subcommands().find(|c| c.get_name() == *name);
            let Some(found) = found else {
                return Err(Fault::UnknownCommand {
                    command: title,
                    commands: listed(&tree).into_iter().map(|c| c.title).collect(),
                }
                .into());
            };
            command = found;
        }
        let mut faults = Faults::default();
        let answer = self.alone(&title, command, &mut faults);
        faults.or(answer)
    }

    /// One topic's Markdown, what `<binary> help <name>` answers. A topic
    /// that does not exist is an error naming those that do.
    pub fn topic_help(&self, name: &str) -> Result<String, Error> {
        let Some(topic) = self.topics.iter().find(|t| t.reference.topic() == name) else {
            return Err(Fault::UnknownTopic {
                topic: name.to_owned(),
                topics: self.topics.iter().map(|t| t.reference.topic()).collect(),
            }
            .into());
        };
        let mut faults = Faults::default();
        let text = self.topic_text(topic, &mut faults);
        faults.or(format!("# `{}`\n\n{text}", topic.reference))
    }

    /// The tool's command, every `--help` and `-h` in it answering with this
    /// manual: the tool's with the whole manual, each subcommand's with its
    /// own text. Checks every command, hidden ones too, so a tool refuses to
    /// start with a broken manual rather than show one.
    pub fn clap(&self) -> Result<Command, Error> {
        let mut faults = Faults::default();
        let tree = self.tree();
        let manual = self.whole(&tree, &mut faults);
        let command = self.answering(
            self.command.clone().override_help(manual),
            &tree,
            self.command.get_name(),
            &mut faults,
        );
        faults.or(command)
    }

    /// `command`'s subcommands, at every depth, each answering `--help` with
    /// its own text, read from `tree`, the same command in [`Help::tree`].
    fn answering(
        &self,
        mut command: Command,
        tree: &Command,
        title: &str,
        faults: &mut Faults,
    ) -> Command {
        for built in tree.get_subcommands() {
            command = command.mut_subcommand(built.get_name(), |sub| {
                let title = format!("{title} {}", built.get_name());
                let text = self.alone(&title, built, faults);
                self.answering(sub.override_help(text), built, &title, faults)
            });
        }
        command
    }

    /// The command tree as clap parses it, which the manual reads: global
    /// arguments reach every command they apply to, `--help` and usage are
    /// clap's, and no `help` subcommand is added, so only the tool's own
    /// commands are there.
    fn tree(&self) -> Command {
        fn own(command: Command) -> Command {
            let names: Vec<String> = command
                .get_subcommands()
                .map(|c| c.get_name().to_owned())
                .collect();
            let mut command = command.disable_help_subcommand(true);
            for name in names {
                command = command.mut_subcommand(name, own);
            }
            command
        }
        let mut tree = own(self.command.clone());
        tree.build();
        tree
    }

    /// The whole manual of `tree`, [`Help::tree`].
    fn whole(&self, tree: &Command, faults: &mut Faults) -> String {
        let root = tree.get_name();
        let mut manual = format!("# `{root}`\n");
        manual.push_str(&self.body(&format!("`{root}`"), tree, 2, faults));

        let listed = listed(tree);
        let mut groups: Vec<&str> = Vec::new();
        for command in &listed {
            match command.group {
                Some(group) if !groups.contains(&group) => groups.push(group),
                Some(_) => {}
                None => faults.push(Fault::UngroupedCommand {
                    command: command.title.clone(),
                }),
            }
        }
        if !listed.is_empty() {
            manual.push_str("\n## Commands\n");
        }
        for group in groups {
            manual.push_str(&format!("\n### {group}\n"));
            for command in listed.iter().filter(|c| c.group == Some(group)) {
                manual.push_str(&format!("\n#### `{}`\n", command.title));
                manual.push_str(&self.body(
                    &format!("`{}`", command.title),
                    command.command,
                    5,
                    faults,
                ));
            }
        }

        if !self.topics.is_empty() {
            manual.push_str("\n## Topics\n");
        }
        for topic in &self.topics {
            manual.push_str(&format!("\n### `{}`\n\n", topic.reference));
            manual.push_str(&self.topic_text(topic, faults));
        }
        manual
    }

    /// One command's answer: its heading, then its body.
    fn alone(&self, title: &str, command: &Command, faults: &mut Faults) -> String {
        format!(
            "# `{title}`\n{}",
            self.body(&format!("`{title}`"), command, 2, faults)
        )
    }

    /// What the manual says of one command of [`Help::tree`], under its
    /// heading: its text, then its usage and its arguments, each under a
    /// heading `depth` deep. Starts with a blank line and ends with a newline.
    fn body(&self, at: &str, command: &Command, depth: usize, faults: &mut Faults) -> String {
        let heading = "#".repeat(depth);
        let mut body = self.text(at, command, faults);
        let usage = command.clone().render_usage().to_string();
        let usage = usage.strip_prefix("Usage: ").unwrap_or(&usage);
        body.push_str(&format!("\n{heading} Usage\n\n```\n{usage}\n```\n"));
        let arguments = self.arguments(at, command, faults);
        if !arguments.is_empty() {
            body.push_str(&format!("\n{heading} Arguments\n\n{arguments}"));
        }
        body
    }

    /// A command's visible arguments as a Markdown list, positional ones
    /// first, then the rest in clap's order: each named as clap's usage names
    /// it, with its help, then its default, its environment variable and the
    /// values it accepts, where set. Empty when it takes none.
    fn arguments(&self, at: &str, command: &Command, faults: &mut Faults) -> String {
        let visible = || {
            command.get_arguments().filter(|arg| {
                let clap_s_own = matches!(
                    arg.get_action(),
                    ArgAction::Help
                        | ArgAction::HelpShort
                        | ArgAction::HelpLong
                        | ArgAction::Version
                );
                !clap_s_own && !arg.is_hide_set() && !arg.is_hide_long_help_set()
            })
        };
        let mut positional: Vec<&Arg> = visible().filter(|arg| arg.is_positional()).collect();
        positional.sort_by_key(|arg| arg.get_index());
        let options = visible().filter(|arg| !arg.is_positional());

        let mut list = String::new();
        for arg in positional.into_iter().chain(options) {
            let name = match (arg.get_short(), arg.get_long()) {
                (Some(short), Some(_)) => format!("-{short}, {arg}"),
                _ => arg.to_string(),
            };
            let at = format!("`{name}` of {at}");
            let help = arg.get_long_help().or(arg.get_help());
            let help = help.map(|help| self.expand(&help.to_string(), &at, false, faults));
            item(
                &mut list,
                "",
                &name,
                help.as_deref().unwrap_or("").trim_end(),
            );

            let takes_values = arg.get_action().takes_values();
            let defaults = arg.get_default_values();
            if takes_values && !defaults.is_empty() && !arg.is_hide_default_value_set() {
                let defaults: Vec<_> = defaults.iter().map(|d| d.to_string_lossy()).collect();
                list.push_str(&format!("  - Default: {}\n", self::list(&defaults)));
            }
            if let Some(env) = arg.get_env().filter(|_| !arg.is_hide_env_set()) {
                list.push_str(&format!("  - Environment: `{}`\n", env.to_string_lossy()));
            }
            let values: Vec<_> = arg
                .get_possible_values()
                .into_iter()
                .filter(|value| !value.is_hide_set())
                .collect();
            if takes_values && !values.is_empty() && !arg.is_hide_possible_values_set() {
                list.push_str("  - Values:\n");
                for value in values {
                    let help = value.get_help().map(ToString::to_string);
                    let help = help.map(|help| self.expand(&help, &at, false, faults));
                    item(
                        &mut list,
                        "    ",
                        value.get_name(),
                        help.as_deref().unwrap_or("").trim_end(),
                    );
                }
            }
        }
        list
    }

    /// A command's text, placeholders replaced, as a block that starts with
    /// a blank line and ends with a newline; empty when it has none.
    fn text(&self, at: &str, command: &Command, faults: &mut Faults) -> String {
        let text = command.get_long_about().or(command.get_about());
        match text.map(ToString::to_string) {
            Some(text) if !text.trim().is_empty() => {
                format!("\n{}\n", self.expand(&text, at, false, faults).trim_end())
            }
            _ => String::new(),
        }
    }

    fn topic_text(&self, topic: &RegisteredTopic, faults: &mut Faults) -> String {
        let tool = self.command.get_name();
        if topic.reference.binary() != tool {
            faults.push(Fault::ForeignTopic {
                reference: topic.reference,
                tool: tool.to_owned(),
            });
        }
        let at = format!("topic `{}`", topic.reference.topic());
        format!(
            "{}\n",
            self.expand(topic.markdown, &at, false, faults).trim_end()
        )
    }

    /// `text` with each `{table:<name>}` and `{const:<NAME>}` replaced; `at`
    /// says where it is written, for a fault to name.
    fn expand(&self, text: &str, at: &str, in_table: bool, faults: &mut Faults) -> String {
        let mut expanded = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(start) = rest.find('{') {
            expanded.push_str(&rest[..start]);
            let Some((placeholder, after)) = Placeholder::parse(&rest[start..]) else {
                expanded.push('{');
                rest = &rest[start + 1..];
                continue;
            };
            rest = after;
            match placeholder {
                Placeholder::Table(name) if in_table => faults.push(Fault::TableInTable {
                    table: name.to_owned(),
                    at: at.to_owned(),
                }),
                Placeholder::Table(name) => {
                    match self.tables.iter().find(|table| table.name == name) {
                        Some(table) => {
                            let at = format!("table `{}`", table.name);
                            let list = self.expand(&(table.markdown)(), &at, true, faults);
                            expanded.push_str(list.trim_end_matches('\n'));
                        }
                        None => faults.push(Fault::UnknownTable {
                            table: name.to_owned(),
                            at: at.to_owned(),
                            tables: self.tables.iter().map(|table| table.name).collect(),
                        }),
                    }
                }
                Placeholder::Constant(name) => {
                    match self
                        .constants
                        .iter()
                        .find(|(constant, _)| *constant == name)
                    {
                        Some((_, value)) => expanded.push_str(value),
                        None => faults.push(Fault::UnknownConstant {
                            constant: name.to_owned(),
                            at: at.to_owned(),
                            constants: self.constants.iter().map(|(name, _)| *name).collect(),
                        }),
                    }
                }
            }
        }
        expanded.push_str(rest);
        expanded
    }
}

/// Every visible subcommand of `tree`, depth first in clap's order, with its
/// group.
fn listed(tree: &Command) -> Vec<Listed<'_>> {
    fn walk<'a>(
        command: &'a Command,
        title: &str,
        group: Option<&'a str>,
        listed: &mut Vec<Listed<'a>>,
    ) {
        for sub in command.get_subcommands().filter(|c| !c.is_hide_set()) {
            let title = format!("{title} {}", sub.get_name());
            let group = sub.get_next_help_heading().or(group);
            listed.push(Listed {
                title: title.clone(),
                command: sub,
                group,
            });
            walk(sub, &title, group, listed);
        }
    }
    let mut listed = Vec::new();
    walk(tree, tree.get_name(), None, &mut listed);
    listed
}

/// A command in the manual's list.
struct Listed<'a> {
    /// The command as it is run, such as `plans plan create`.
    title: String,
    command: &'a Command,
    /// Its own `next_help_heading`, else the nearest parent's.
    group: Option<&'a str>,
}

enum Placeholder<'a> {
    Table(&'a str),
    Constant(&'a str),
}

impl<'a> Placeholder<'a> {
    /// The placeholder `text` starts with, and the text after it. A name is
    /// one or more characters, none of them whitespace or a brace.
    fn parse(text: &'a str) -> Option<(Self, &'a str)> {
        let (kind, rest) = text.strip_prefix('{')?.split_once(':')?;
        let (name, after) = rest.split_once('}')?;
        if name.is_empty() || name.contains(|c: char| c.is_whitespace() || c == '{') {
            return None;
        }
        let placeholder = match kind {
            "table" => Placeholder::Table(name),
            "const" => Placeholder::Constant(name),
            _ => return None,
        };
        Some((placeholder, after))
    }
}

/// Faults found while answering, each once, in the order found.
#[derive(Default)]
struct Faults(Vec<Fault>);

impl Faults {
    fn push(&mut self, fault: Fault) {
        if !self.0.contains(&fault) {
            self.0.push(fault);
        }
    }

    /// `answer`, unless a fault was found.
    fn or<T>(self, answer: T) -> Result<T, Error> {
        if self.0.is_empty() {
            Ok(answer)
        } else {
            Err(Error { faults: self.0 })
        }
    }
}

/// Why a manual could not answer: every fault it found, each a sentence that
/// says how to fix it, one per line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    faults: Vec<Fault>,
}

impl Error {
    /// Every fault, in the order found; never empty.
    pub fn faults(&self) -> &[Fault] {
        &self.faults
    }
}

impl From<Fault> for Error {
    fn from(fault: Fault) -> Self {
        Self {
            faults: vec![fault],
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, fault) in self.faults.iter().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            write!(f, "{fault}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

/// One thing wrong with a manual, or with what it was asked.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Fault {
    /// A visible subcommand, and every parent of it, declares no group.
    UngroupedCommand {
        /// As it is run, such as `plans serve`.
        command: String,
    },
    /// `{table:<table>}` names no registered table.
    UnknownTable {
        table: String,
        /// Where it is written, such as `` `plans plan create` ``.
        at: String,
        /// The tables there are.
        tables: Vec<&'static str>,
    },
    /// `{const:<constant>}` names no registered constant.
    UnknownConstant {
        constant: String,
        /// Where it is written.
        at: String,
        /// The constants there are.
        constants: Vec<&'static str>,
    },
    /// `{table:<table>}` is written in a table's help.
    TableInTable {
        table: String,
        /// The table whose help it is in.
        at: String,
    },
    /// A topic's crate named a binary other than the tool's command.
    ForeignTopic { reference: TopicRef, tool: String },
    /// A command asked for that does not exist.
    UnknownCommand {
        command: String,
        /// The commands there are, hidden ones left out.
        commands: Vec<String>,
    },
    /// A topic asked for that does not exist.
    UnknownTopic {
        topic: String,
        /// The topics there are.
        topics: Vec<&'static str>,
    },
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::UngroupedCommand { command } => write!(
                f,
                "`{command}` has no group: declare who runs it where it is defined, \
                 with `#[command(next_help_heading = \"...\")]`"
            ),
            Fault::UnknownTable { table, at, tables } => write!(
                f,
                "`{{table:{table}}}` in {at} names no table; the tables are: {}",
                list(tables)
            ),
            Fault::UnknownConstant {
                constant,
                at,
                constants,
            } => write!(
                f,
                "`{{const:{constant}}}` in {at} names no constant; the constants are: {}",
                list(constants)
            ),
            Fault::TableInTable { table, at } => write!(
                f,
                "`{{table:{table}}}` in {at}: a table's help names constants, not tables"
            ),
            Fault::ForeignTopic { reference, tool } => write!(
                f,
                "topic `{}` is shown by `{reference}`, but this tool is `{tool}`: \
                 name it `agent_help::binary!(\"{tool}\")`",
                reference.topic()
            ),
            Fault::UnknownCommand { command, commands } => write!(
                f,
                "no command `{command}`; the commands are: {}",
                list(commands)
            ),
            Fault::UnknownTopic { topic, topics } => {
                write!(f, "no topic `{topic}`; the topics are: {}", list(topics))
            }
        }
    }
}

/// `names` in backticks, joined by commas; `none` when there are none.
fn list(names: &[impl AsRef<str>]) -> String {
    if names.is_empty() {
        return "none".to_owned();
    }
    names
        .iter()
        .map(|name| format!("`{}`", name.as_ref()))
        .collect::<Vec<_>>()
        .join(", ")
}
