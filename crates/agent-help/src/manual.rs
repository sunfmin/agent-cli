use std::fmt;

use clap::builder::StyledStr;
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
/// - A command's or topic's text is written as a page of its own: its
///   sections are `##` headings, under the `#` title the manual writes. Asked
///   for alone, it is kept as written; in the whole manual, under
///   ``#### `tool plan create` `` or ``### `tool help sign-in` ``, each
///   heading moves down so `##` sits one level below that heading, a table's
///   help with the text it is in. Lines in a code block fenced with
///   ```` ``` ```` or `~~~` are not headings. A `#` heading in a text is an
///   error, and so is one the manual would move past `######`: write a
///   command's sections at most `###` deep, a topic's at most `####`.
/// - Every subcommand declares its group, who runs it, where it is defined:
///   `#[command(next_help_heading = "Run by the operator")]`. A nested one
///   is in its parent's group unless it declares its own; a subcommand with
///   none is an error.
/// - Inside any text, `{table:<name>}` is the table registered under that
///   name, as [`Table::markdown`] writes it, and `{const:<NAME>}` the value
///   registered under that name: a number is written once, in code. A table's
///   help may name constants, but not tables. One naming nothing registered
///   is an error.
/// - Inside any text, `[...](help:<topic>)` links to a topic; one naming no
///   registered topic is an error. Two topics may not share a name.
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
///   level down (ADR 0005).
/// - The index, what the tool's `--help` answers, is the tool's own answer,
///   then one line per command, the first line of its text, and one per
///   topic, the first sentence of its Markdown; the whole manual is one
///   command away, `<binary> help all`, and `all` is never a topic's name.
/// - Hidden commands, hidden arguments, hidden values, and clap's own
///   `--help` and `--version` are left out of the manual.
/// - [`check`] is the one test a tool needs to keep this manual whole, and
///   [`code_blocks`] hands it the manual's examples to compile.
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

impl RegisteredTopic {
    /// Where its text is written, for a fault or a code block to name.
    fn at(&self) -> String {
        format!("topic `{}`", self.reference.topic())
    }
}

/// A table, as [`Help::table`] registered it.
// `Copy`, so the derived `Clone` copies it whole: one that clones the `fn`
// field crashes Flux (`invalid builtin assoc reft ... Clone ... "cloned"`).
#[derive(Clone, Copy, Debug)]
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

    /// The index, what `<binary> --help` answers: the tool's own text, then
    /// each command's line under its group (groups in the order the whole
    /// manual has them), then each topic's line, and last `<binary> help
    /// all`'s. A command's line is the first line of its `about`, else of its
    /// `long_about`; a topic's is the first sentence of its Markdown, as
    /// [`check`] requires it to open. It answers only when the whole manual
    /// does, with the same faults.
    pub fn index(&self) -> Result<String, Error> {
        let mut faults = Faults::default();
        let tree = self.tree();
        self.whole(&tree, &mut faults);
        let index = self.indexed(&tree, &mut faults);
        faults.or(index)
    }

    /// The whole manual, what `<binary> help all` answers: the tool's own
    /// text, then every command under its group (groups in the order clap's
    /// tree first names them), then every topic.
    pub fn manual(&self) -> Result<String, Error> {
        let mut faults = Faults::default();
        let manual = self.whole(&self.tree(), &mut faults);
        faults.or(manual)
    }

    /// One command's text, what `<binary> <path...> --help` answers; the
    /// empty path is the tool itself, whose answer is the [index](Help::index).
    /// A command that does not exist is an error naming those that do.
    pub fn command_help(&self, path: &[&str]) -> Result<String, Error> {
        if path.is_empty() {
            return self.index();
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

    /// One topic's Markdown, what `<binary> help <name>` answers; `all` is
    /// the [whole manual](Help::manual). A topic that does not exist is an
    /// error naming those that do.
    pub fn topic_help(&self, name: &str) -> Result<String, Error> {
        if name == ALL {
            return self.manual();
        }
        let Some(topic) = self.topics.iter().find(|t| t.reference.topic() == name) else {
            return Err(Fault::UnknownTopic {
                topic: name.to_owned(),
                topics: self.topics.iter().map(|t| t.reference.topic()).collect(),
            }
            .into());
        };
        let mut faults = Faults::default();
        let text = self.topic_text(topic, PAGE, &mut faults);
        let title = heading(PAGE, &format!("`{}`", topic.reference));
        faults.or(format!("{title}\n{text}"))
    }

    /// The tool's command, every `--help` and `-h` in it answering from this
    /// manual: the tool's with the index, each subcommand's with its own
    /// text. Checks the whole manual and every command, hidden ones too, so a
    /// tool refuses to start with a broken manual rather than show one.
    pub fn clap(&self) -> Result<Command, Error> {
        let mut faults = Faults::default();
        let tree = self.tree();
        self.whole(&tree, &mut faults);
        let index = self.indexed(&tree, &mut faults);
        let command = self.answering(
            self.command.clone().override_help(index),
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
        let mut manual = heading(PAGE, &format!("`{root}`"));
        manual.push_str(&self.body(&format!("`{root}`"), tree, PAGE, faults));

        let listed = listed(tree);
        if !listed.is_empty() {
            manual.push_str("\n## Commands\n");
        }
        for group in groups(&listed, faults) {
            manual.push_str(&format!("\n### {group}\n"));
            for command in listed.iter().filter(|c| c.group == Some(group)) {
                let at = format!("`{}`", command.title);
                manual.push('\n');
                manual.push_str(&heading(COMMAND, &at));
                manual.push_str(&self.body(&at, command.command, COMMAND, faults));
            }
        }

        if !self.topics.is_empty() {
            manual.push_str("\n## Topics\n");
        }
        for (i, topic) in self.topics.iter().enumerate() {
            let name = topic.reference.topic();
            if self.topics[..i].iter().any(|t| t.reference.topic() == name) {
                faults.push(Fault::DuplicateTopic {
                    topic: name.to_owned(),
                });
            }
        }
        for topic in &self.topics {
            manual.push('\n');
            manual.push_str(&heading(TOPIC, &format!("`{}`", topic.reference)));
            manual.push('\n');
            manual.push_str(&self.topic_text(topic, TOPIC, faults));
        }
        manual
    }

    /// The index of `tree`, [`Help::tree`].
    fn indexed(&self, tree: &Command, faults: &mut Faults) -> String {
        let root = tree.get_name();
        let mut index = heading(PAGE, &format!("`{root}`"));
        index.push_str(&self.body(&format!("`{root}`"), tree, PAGE, faults));

        let listed = listed(tree);
        if !listed.is_empty() {
            index.push_str("\n## Commands\n");
        }
        for group in groups(&listed, faults) {
            index.push_str(&format!("\n### {group}\n\n"));
            for command in listed.iter().filter(|c| c.group == Some(group)) {
                let line = command
                    .command
                    .get_about()
                    .or(command.command.get_long_about())
                    .map(|about| {
                        let at = format!("`{}`", command.title);
                        let about = self.expand(&about.to_string(), &at, false, faults);
                        about.lines().next().unwrap_or_default().trim().to_owned()
                    })
                    .unwrap_or_default();
                item(&mut index, "", &command.title, &line);
            }
        }

        index.push_str("\n## Topics\n\n");
        for topic in &self.topics {
            let markdown = self.expand(topic.markdown, &topic.at(), false, faults);
            let line = sentence(&markdown).unwrap_or_default();
            item(&mut index, "", &topic.reference.to_string(), line);
        }
        item(
            &mut index,
            "",
            &format!("{root} help {ALL}"),
            "The whole manual: every command and every topic, in full.",
        );
        index
    }

    /// One command's answer: its heading, then its body.
    fn alone(&self, title: &str, command: &Command, faults: &mut Faults) -> String {
        let at = format!("`{title}`");
        let body = self.body(&at, command, PAGE, faults);
        format!("{}{body}", heading(PAGE, &at))
    }

    /// What the manual says of one command of [`Help::tree`] under its
    /// heading, of level `under`: its text, then its usage and its arguments,
    /// each a `##` section as on a page of its own, every heading [`nest`]ed.
    /// Starts with a blank line and ends with a newline.
    fn body(&self, at: &str, command: &Command, under: usize, faults: &mut Faults) -> String {
        let mut body = self.text(at, command, faults);
        let usage = command.clone().render_usage().to_string();
        let usage = usage.strip_prefix("Usage: ").unwrap_or(&usage);
        body.push_str(&format!("\n## Usage\n\n```\n{usage}\n```\n"));
        let arguments = self.arguments(at, command, faults);
        if !arguments.is_empty() {
            body.push_str(&format!("\n## Arguments\n\n{arguments}"));
        }
        nest(&body, under, at, faults)
    }

    /// A command's visible arguments as a Markdown list, in the order
    /// [`visible`] gives: each named as clap's usage names it, with its help,
    /// then its default, its environment variable and the values it accepts,
    /// where set. Empty when it takes none.
    fn arguments(&self, at: &str, command: &Command, faults: &mut Faults) -> String {
        let mut list = String::new();
        for (name, arg) in visible(command) {
            let (help, values) = self.argument_help(&format!("`{name}` of {at}"), arg, faults);
            item(&mut list, "", &name, &help);
            let defaults = arg.get_default_values();
            if arg.get_action().takes_values()
                && !defaults.is_empty()
                && !arg.is_hide_default_value_set()
            {
                let defaults: Vec<_> = defaults.iter().map(|d| d.to_string_lossy()).collect();
                list.push_str(&format!("  - Default: {}\n", self::list(&defaults)));
            }
            if let Some(env) = arg.get_env().filter(|_| !arg.is_hide_env_set()) {
                list.push_str(&format!("  - Environment: `{}`\n", env.to_string_lossy()));
            }
            if !values.is_empty() {
                list.push_str("  - Values:\n");
                for (value, help) in values {
                    item(&mut list, "    ", &value, &help);
                }
            }
        }
        list
    }

    /// `arg`'s help, its `long_help` else its `help`, and each value it
    /// accepts that the manual lists, with that value's help: placeholders
    /// replaced, each help empty when there is none. `at` names the argument,
    /// for a fault to name.
    fn argument_help(
        &self,
        at: &str,
        arg: &Arg,
        faults: &mut Faults,
    ) -> (String, Vec<(String, String)>) {
        let mut help = |text: Option<&StyledStr>| match text {
            Some(text) => self
                .expand(&text.to_string(), at, false, faults)
                .trim_end()
                .to_owned(),
            None => String::new(),
        };
        let own = help(arg.get_long_help().or(arg.get_help()));
        let mut values = Vec::new();
        if arg.get_action().takes_values() && !arg.is_hide_possible_values_set() {
            for value in arg.get_possible_values() {
                if !value.is_hide_set() {
                    values.push((value.get_name().to_owned(), help(value.get_help())));
                }
            }
        }
        (own, values)
    }

    /// A command's text, placeholders replaced, as a block that starts with
    /// a blank line and ends with a newline; empty when it has none.
    fn text(&self, at: &str, command: &Command, faults: &mut Faults) -> String {
        match written(command) {
            Some(text) => format!("\n{}\n", self.expand(&text, at, false, faults).trim_end()),
            None => String::new(),
        }
    }

    /// A topic's text under a heading of level `under`, placeholders replaced
    /// and headings nested, ended by a newline.
    fn topic_text(&self, topic: &RegisteredTopic, under: usize, faults: &mut Faults) -> String {
        let tool = self.command.get_name();
        if topic.reference.binary() != tool {
            faults.push(Fault::ForeignTopic {
                reference: topic.reference,
                tool: tool.to_owned(),
            });
        }
        let at = topic.at();
        let text = nest(
            &self.expand(topic.markdown, &at, false, faults),
            under,
            &at,
            faults,
        );
        format!("{}\n", text.trim_end())
    }

    /// `text` with each `{table:<name>}` and `{const:<NAME>}` replaced; `at`
    /// says where it is written, for a fault to name.
    fn expand(&self, text: &str, at: &str, in_table: bool, faults: &mut Faults) -> String {
        for topic in links(text) {
            if !self.topics.iter().any(|t| t.reference.topic() == topic) {
                faults.push(Fault::UnknownLink {
                    topic: topic.to_owned(),
                    at: at.to_owned(),
                    topics: self.topics.iter().map(|t| t.reference.topic()).collect(),
                });
            }
        }
        let mut expanded = String::with_capacity(text.len());
        let mut rest = text;
        while let Some((before, inside)) = rest.split_once('{') {
            expanded.push_str(before);
            let Some((placeholder, after)) = Placeholder::parse(inside) else {
                expanded.push('{');
                rest = inside;
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

/// `command`'s arguments the manual lists, each with its name as clap's usage
/// names it, such as `-s, --store <PATH>` or `<NAME>`: positional ones first,
/// by index, then the rest in clap's order. Hidden ones, and clap's own
/// `--help` and `--version`, are left out.
fn visible(command: &Command) -> Vec<(String, &Arg)> {
    let mut positional = Vec::new();
    let mut options = Vec::new();
    for arg in command.get_arguments() {
        let clap_s_own = matches!(
            arg.get_action(),
            ArgAction::Help | ArgAction::HelpShort | ArgAction::HelpLong | ArgAction::Version
        );
        if clap_s_own || arg.is_hide_set() || arg.is_hide_long_help_set() {
            continue;
        }
        if arg.is_positional() {
            positional.push(arg);
        } else {
            options.push(arg);
        }
    }
    positional.sort_by_key(|arg| arg.get_index());
    let mut visible = Vec::new();
    for arg in positional.into_iter().chain(options) {
        let name = match (arg.get_short(), arg.get_long()) {
            (Some(short), Some(_)) => format!("-{short}, {arg}"),
            _ => arg.to_string(),
        };
        visible.push((name, arg));
    }
    visible
}

/// `tree` and every subcommand of it, hidden ones too, depth first in
/// clap's order, each with its title, such as `plans plan create`.
fn every(tree: &Command) -> Vec<(String, &Command)> {
    fn walk<'a>(command: &'a Command, title: String, every: &mut Vec<(String, &'a Command)>) {
        every.push((title.clone(), command));
        for sub in command.get_subcommands() {
            walk(sub, format!("{title} {}", sub.get_name()), every);
        }
    }
    let mut every = Vec::new();
    walk(tree, tree.get_name().to_owned(), &mut every);
    every
}

/// The groups of `listed`, in the order it first names them. A command with
/// no group is a fault naming it.
fn groups<'a>(listed: &[Listed<'a>], faults: &mut Faults) -> Vec<&'a str> {
    let mut groups: Vec<&str> = Vec::new();
    for command in listed {
        match command.group {
            Some(group) if !groups.contains(&group) => groups.push(group),
            Some(_) => {}
            None => faults.push(Fault::UngroupedCommand {
                command: command.title.clone(),
            }),
        }
    }
    groups
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

/// Checks `help` whole, the one test a tool needs for its manual: every
/// command, hidden ones too, has text and a group, every placeholder and
/// every `help:` link names something registered, no two topics share a
/// name, every topic opens with a sentence, its line in the index, and none
/// is named `all`, which `<binary> help all` answers with the whole manual.
/// Fails naming every fault at once.
///
/// ```no_run
/// # fn help() -> agent_help::Help { unimplemented!() }
/// #[test]
/// fn the_manual_is_whole() {
///     agent_help::check(&help()).unwrap();
/// }
/// ```
pub fn check(help: &Help) -> Result<(), Error> {
    let mut faults = match help.clap() {
        Ok(_) => Faults::default(),
        Err(error) => Faults(error.faults),
    };
    for (title, command) in every(&help.tree()) {
        if written(command).is_none() {
            faults.push(Fault::NoText { command: title });
        }
    }
    for topic in &help.topics {
        if topic.reference.topic() == ALL {
            faults.push(Fault::TopicNamedAll {
                reference: topic.reference,
            });
        }
        if sentence(topic.markdown).is_none() {
            faults.push(Fault::NoSentence {
                topic: topic.reference.topic().to_owned(),
            });
        }
    }
    faults.or(())
}

/// Every fenced code block of `language` in `help`, as its answers show it
/// (placeholders replaced, a table's indent removed), so a tool can compile
/// or run the examples its manual shows: each command's in clap's order,
/// hidden ones too, its text's then its arguments' (`at` naming the
/// argument), then each topic's. A block's language is the first word
/// of its info string, so `rust,no_run` is `rust`.
pub fn code_blocks(help: &Help, language: &str) -> Vec<CodeBlock> {
    // Faults are `check`'s to report.
    let mut faults = Faults::default();
    let mut texts = Vec::new();
    for (title, command) in every(&help.tree()) {
        let at = format!("`{title}`");
        texts.push((at.clone(), help.text(&at, command, &mut faults)));
        for (name, arg) in visible(command) {
            let at = format!("`{name}` of {at}");
            let (own, values) = help.argument_help(&at, arg, &mut faults);
            texts.push((at.clone(), own));
            texts.extend(values.into_iter().map(|(_, value)| (at.clone(), value)));
        }
    }
    for topic in &help.topics {
        texts.push((topic.at(), help.topic_text(topic, PAGE, &mut faults)));
    }
    let mut blocks = Vec::new();
    for (at, text) in texts {
        for code in fenced(&text, language) {
            blocks.push(CodeBlock {
                at: at.clone(),
                code,
            });
        }
    }
    blocks
}

/// A fenced code block of a manual, as [`code_blocks`] hands it over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeBlock {
    /// Where it is shown, such as `` `plans plan create` `` or
    /// `` topic `sign-in` ``.
    pub at: String,
    /// Its lines, each ended by a newline.
    pub code: String,
}

/// A command's text, its `long_about` else its `about`; none when blank.
fn written(command: &Command) -> Option<String> {
    let text = command
        .get_long_about()
        .or(command.get_about())?
        .to_string();
    (!text.trim().is_empty()).then_some(text)
}

/// The topic each `[...](help:<topic>)` in `text` names. A topic is written
/// without whitespace; the empty one names none.
fn links(text: &str) -> impl Iterator<Item = &str> {
    text.split("](help:").skip(1).filter_map(|rest| {
        let (topic, _) = rest.split_once(')')?;
        (!topic.contains(|c: char| c.is_whitespace() || c == '(')).then_some(topic)
    })
}

/// The first sentence of `markdown`, the line the index shows of a topic: its
/// first line, which opens a paragraph, up to the first `.`, `!` or `?`
/// outside a code span that ends the line or is followed by a space. None
/// when `markdown` does not open with one.
fn sentence(markdown: &str) -> Option<&str> {
    let line = markdown.lines().next()?;
    let numbered = line.trim_start_matches(|c: char| c.is_ascii_digit());
    let opens_a_block = line.is_empty()
        || line.starts_with([' ', '\t', '>', '|', '<'])
        || atx(line).is_some()
        || Fence::opened_by(line).is_some()
        || ["- ", "* ", "+ "]
            .iter()
            .any(|marker| line.starts_with(marker))
        || (numbered.len() < line.len()
            && (numbered.starts_with(". ") || numbered.starts_with(") ")));
    if opens_a_block {
        return None;
    }
    let mut code = false;
    let mut chars = line.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        match c {
            '`' => code = !code,
            '.' | '!' | '?' if !code && matches!(chars.peek(), None | Some((_, ' '))) => {
                return line.get(..=at);
            }
            _ => {}
        }
    }
    None
}

/// What `<binary> help` answers with the whole manual, in place of a topic.
const ALL: &str = "all";

/// The level of the heading an answer gives the text it asks for alone: a
/// page's title.
const PAGE: usize = 1;
/// The level of a command's heading in the whole manual.
const COMMAND: usize = 4;
/// The level of a topic's heading in the whole manual.
const TOPIC: usize = 3;
/// The deepest heading Markdown has.
const DEEPEST: usize = 6;

/// A heading of `level` naming `title`, ended by a newline.
fn heading(level: usize, title: &str) -> String {
    format!("{} {title}\n", "#".repeat(level))
}

/// `text`, written as a page of its own (its sections from `##`), placed
/// under a heading of level `under`: each heading outside a fenced code block
/// moved down `under - 1` levels, so `##` sits one below `under`. A `#`
/// heading, the title, and one moved past `######` are faults naming `at`.
fn nest(text: &str, under: usize, at: &str, faults: &mut Faults) -> String {
    let shift = under.saturating_sub(PAGE);
    let mut nested = String::with_capacity(text.len());
    let mut fence: Option<Fence> = None;
    for line in text.lines() {
        match &fence {
            Some(open) => {
                if open.closed_by(line) {
                    fence = None;
                }
            }
            None => fence = Fence::opened_by(line),
        }
        match (&fence, atx(line)) {
            (None, Some((indent, level))) => {
                let (before, run) = line.split_at(indent);
                let after = &run[level..];
                if level == PAGE {
                    faults.push(Fault::TitleInText {
                        heading: line.trim().to_owned(),
                        at: at.to_owned(),
                    });
                }
                match level.checked_add(shift).filter(|&l| l <= DEEPEST) {
                    Some(moved) => {
                        nested.push_str(before);
                        nested.push_str(&"#".repeat(moved));
                        nested.push_str(after);
                    }
                    None => {
                        faults.push(Fault::HeadingTooDeep {
                            heading: line.trim().to_owned(),
                            at: at.to_owned(),
                            deepest: DEEPEST.saturating_sub(shift),
                        });
                        nested.push_str(line);
                    }
                }
            }
            _ => nested.push_str(line),
        }
        nested.push('\n');
    }
    nested
}

/// The indent of the ATX heading `line` opens and its level, the length of
/// its `#` run; none when `line` opens no heading. A heading is indented by
/// at most three spaces, and its run of one to six `#` ends the line or is
/// followed by a space or a tab.
fn atx(line: &str) -> Option<(usize, usize)> {
    let indent = leading(line, b' ');
    if indent > 3 {
        return None;
    }
    let run = &line[indent..];
    let level = leading(run, b'#');
    if level == 0 || level > DEEPEST {
        return None;
    }
    match run[level..].bytes().next() {
        None | Some(b' ' | b'\t') => Some((indent, level)),
        Some(_) => None,
    }
}

/// The opening line of a fenced code block: three or more backticks or
/// tildes.
struct Fence {
    /// The spaces before it.
    indent: usize,
    /// `` ` `` or `~`.
    mark: u8,
    /// How many marks it has.
    length: usize,
}

impl Fence {
    /// The fence `line` opens, none when it opens none.
    fn opened_by(line: &str) -> Option<Self> {
        let indent = leading(line, b' ');
        let mark = line.as_bytes().get(indent).copied()?;
        if mark != b'`' && mark != b'~' {
            return None;
        }
        let length = leading(&line[indent..], mark);
        (length >= 3).then_some(Self {
            indent,
            mark,
            length,
        })
    }

    /// The language its info string names, its first word: `rust,no_run`
    /// is `rust`.
    fn language<'a>(&self, opening: &'a str) -> Option<&'a str> {
        let info = opening.trim_start_matches(' ');
        let info = info.trim_start_matches(self.mark as char);
        info.split([',', ' ']).next()
    }

    /// Whether `line` closes the block: only marks of its kind, at least as
    /// many as it opened with.
    fn closed_by(&self, line: &str) -> bool {
        let content = line.trim_start_matches(' ').trim_end();
        content.len() >= self.length && content.bytes().all(|b| b == self.mark)
    }
}

/// The code of each fenced block in `markdown` whose language is `language`,
/// each line less the fence's indent.
fn fenced(markdown: &str, language: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut lines = markdown.lines();
    while let Some(line) = lines.next() {
        let Some(fence) = Fence::opened_by(line) else {
            continue;
        };
        let mut code = String::new();
        for line in lines.by_ref() {
            if fence.closed_by(line) {
                break;
            }
            let strip = leading(line, b' ').min(fence.indent);
            code.push_str(&line[strip..]);
            code.push('\n');
        }
        if fence.language(line) == Some(language) {
            blocks.push(code);
        }
    }
    blocks
}

/// How many bytes `text` starts with that are `byte`, an ASCII byte. Counted
/// rather than taken as the difference of two lengths, which Flux cannot bound.
fn leading(text: &str, byte: u8) -> usize {
    text.bytes().take_while(|&b| b == byte).count()
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
    /// The placeholder whose opening brace `text` follows, and the text after
    /// it. A name is one or more characters, none of them whitespace or a
    /// brace.
    fn parse(text: &'a str) -> Option<(Self, &'a str)> {
        let (kind, rest) = text.split_once(':')?;
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
    /// A command, the tool's own or a subcommand, hidden or not, has no text.
    NoText {
        /// As it is run, such as `plans serve`.
        command: String,
    },
    /// `[...](help:<topic>)` names no registered topic.
    UnknownLink {
        topic: String,
        /// Where it is written.
        at: String,
        /// The topics there are.
        topics: Vec<&'static str>,
    },
    /// Two topics registered under one name.
    DuplicateTopic { topic: String },
    /// A topic's crate named a binary other than the tool's command.
    ForeignTopic { reference: TopicRef, tool: String },
    /// A command asked for that does not exist.
    UnknownCommand {
        command: String,
        /// The commands there are, hidden ones left out.
        commands: Vec<String>,
    },
    /// A `#` heading in a command's or topic's text, where the manual
    /// writes the title.
    TitleInText {
        /// The heading, as written.
        heading: String,
        /// Where it is written.
        at: String,
    },
    /// A heading in a command's or topic's text that the manual would move
    /// past `######`.
    HeadingTooDeep {
        /// The heading, as written.
        heading: String,
        /// Where it is written.
        at: String,
        /// The deepest level it may be written at there.
        deepest: usize,
    },
    /// A topic whose Markdown does not open with a sentence, its line in the
    /// index.
    NoSentence { topic: String },
    /// A topic named `all`, which `<binary> help all` answers with the whole
    /// manual instead.
    TopicNamedAll { reference: TopicRef },
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
            Fault::NoText { command } => write!(
                f,
                "`{command}` has no text: write its doc comment, and keep it as written \
                 with `#[command(verbatim_doc_comment)]`"
            ),
            Fault::UnknownLink { topic, at, topics } => write!(
                f,
                "`help:{topic}` in {at} names no topic; the topics are: {}",
                list(topics)
            ),
            Fault::DuplicateTopic { topic } => write!(
                f,
                "topic `{topic}` is added twice: give each topic a name of its own"
            ),
            Fault::ForeignTopic { reference, tool } => write!(
                f,
                "topic `{}` is shown by `{reference}`, but this tool is `{tool}`: \
                 name it `agent_help::binary!(\"{tool}\")`",
                reference.topic()
            ),
            Fault::TitleInText { heading, at } => write!(
                f,
                "`{heading}` in {at} is a title, and the manual writes the title: \
                 start the text's sections from `##`"
            ),
            Fault::HeadingTooDeep {
                heading,
                at,
                deepest,
            } => write!(
                f,
                "`{heading}` in {at} is nested past `######` in the manual: \
                 write it `{}` at most",
                "#".repeat(*deepest)
            ),
            Fault::UnknownCommand { command, commands } => write!(
                f,
                "no command `{command}`; the commands are: {}",
                list(commands)
            ),
            Fault::NoSentence { topic } => write!(
                f,
                "topic `{topic}` does not open with a sentence, the line the index shows of it: \
                 start its Markdown with one, ended by `.`, `!` or `?`"
            ),
            Fault::TopicNamedAll { reference } => write!(
                f,
                "topic `{}` is named as `{reference}` is, which answers the whole manual: \
                 give it another name",
                reference.topic()
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
