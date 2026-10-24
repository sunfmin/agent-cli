use std::fmt;

use clap::Command;

use crate::{Table, Topic, TopicRef};

/// A tool's manual, assembled from what is written where it lives: clap's
/// command tree, the topics and the tables. Every answer is Markdown, the
/// text kept as its author wrote it.
///
/// ```
/// use agent_help::{Help, Topic};
/// use clap::Command;
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
///                 .long_about("Creates a plan. It may refuse:\n\n{table:refusal}"),
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
/// "
///     );
///     assert_eq!(
///         help.topic_help("sign-in").unwrap(),
///         "# `plans help sign-in`\n\nRun `plans login` once.\n"
///     );
///     assert!(help.manual().unwrap().starts_with("# `plans`\n\nKeeps plans.\n\n## Commands\n"));
/// }
/// ```
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
/// - Inside any text, `[...](help:<topic>)` links to a topic; one naming no
///   registered topic is an error. Two topics may not share a name.
/// - Hidden commands are left out of the manual.
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
        let manual = self.whole(&mut faults);
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
        let mut command = &self.command;
        for name in path {
            let found = command.get_subcommands().find(|c| c.get_name() == *name);
            let Some(found) = found else {
                return Err(Fault::UnknownCommand {
                    command: title,
                    commands: self.listed().into_iter().map(|c| c.title).collect(),
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
        let manual = self.whole(&mut faults);
        let command = self.answering(
            self.command.clone().override_help(manual),
            self.command.get_name(),
            &mut faults,
        );
        faults.or(command)
    }

    /// `command`'s subcommands, at every depth, each answering `--help` with
    /// its own text.
    fn answering(&self, mut command: Command, title: &str, faults: &mut Faults) -> Command {
        let names: Vec<String> = command
            .get_subcommands()
            .map(|c| c.get_name().to_owned())
            .collect();
        for name in names {
            command = command.mut_subcommand(&name, |sub| {
                let title = format!("{title} {name}");
                let text = self.alone(&title, &sub, faults);
                self.answering(sub.override_help(text), &title, faults)
            });
        }
        command
    }

    fn whole(&self, faults: &mut Faults) -> String {
        let root = self.command.get_name();
        let mut manual = format!("# `{root}`\n");
        manual.push_str(&self.text(&format!("`{root}`"), &self.command, faults));

        let listed = self.listed();
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
                manual.push_str(&self.text(
                    &format!("`{}`", command.title),
                    command.command,
                    faults,
                ));
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
            manual.push_str(&format!("\n### `{}`\n\n", topic.reference));
            manual.push_str(&self.topic_text(topic, faults));
        }
        manual
    }

    /// One command's answer: its heading, then its text.
    fn alone(&self, title: &str, command: &Command, faults: &mut Faults) -> String {
        format!(
            "# `{title}`\n{}",
            self.text(&format!("`{title}`"), command, faults)
        )
    }

    /// Every visible subcommand, depth first in clap's order, with its group.
    fn listed(&self) -> Vec<Listed<'_>> {
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
        walk(&self.command, self.command.get_name(), None, &mut listed);
        listed
    }

    /// A command's text, placeholders replaced, as a block that starts with
    /// a blank line and ends with a newline; empty when it has none.
    fn text(&self, at: &str, command: &Command, faults: &mut Faults) -> String {
        match written(command) {
            Some(text) => format!("\n{}\n", self.expand(&text, at, false, faults).trim_end()),
            None => String::new(),
        }
    }

    /// Every command, hidden ones too, depth first in clap's order from the
    /// tool's own, each with its title, such as `plans plan create`.
    fn every(&self) -> Vec<(String, &Command)> {
        fn walk<'a>(command: &'a Command, title: String, every: &mut Vec<(String, &'a Command)>) {
            every.push((title.clone(), command));
            for sub in command.get_subcommands() {
                walk(sub, format!("{title} {}", sub.get_name()), every);
            }
        }
        let mut every = Vec::new();
        walk(
            &self.command,
            self.command.get_name().to_owned(),
            &mut every,
        );
        every
    }

    fn topic_text(&self, topic: &RegisteredTopic, faults: &mut Faults) -> String {
        let tool = self.command.get_name();
        if topic.reference.binary() != tool {
            faults.push(Fault::ForeignTopic {
                reference: topic.reference,
                tool: tool.to_owned(),
            });
        }
        let at = topic.at();
        format!(
            "{}\n",
            self.expand(topic.markdown, &at, false, faults).trim_end()
        )
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

/// Checks `help` whole, the one test a tool needs for its manual: every
/// command, hidden ones too, has text and a group, every placeholder and
/// every `help:` link names something registered, and no two topics share a
/// name. Fails naming every fault at once.
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
    for (title, command) in help.every() {
        if written(command).is_none() {
            faults.push(Fault::NoText { command: title });
        }
    }
    faults.or(())
}

/// Every fenced code block of `language` in `help`, as its answers show it
/// (placeholders replaced, a table's indent removed), so a tool can compile
/// or run the examples its manual shows: each command's in clap's order,
/// hidden ones too, then each topic's. A block's language is the first word
/// of its info string, so `rust,no_run` is `rust`.
pub fn code_blocks(help: &Help, language: &str) -> Vec<CodeBlock> {
    // Faults are `check`'s to report.
    let mut faults = Faults::default();
    let mut texts = Vec::new();
    for (title, command) in help.every() {
        let at = format!("`{title}`");
        let text = help.text(&at, command, &mut faults);
        texts.push((at, text));
    }
    for topic in &help.topics {
        texts.push((topic.at(), help.topic_text(topic, &mut faults)));
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

/// The code of each block fenced with backticks in `markdown` whose language
/// is `language`, each line less the fence's indent.
fn fenced(markdown: &str, language: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut lines = markdown.lines();
    while let Some(line) = lines.next() {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let opening = &line[indent..];
        let ticks = opening.len() - opening.trim_start_matches('`').len();
        if ticks < 3 {
            continue;
        }
        let info = &opening[ticks..];
        let mut code = String::new();
        for line in lines.by_ref() {
            let content = line.trim_start_matches(' ');
            if content.trim_end().len() >= ticks && content.trim_end().chars().all(|c| c == '`') {
                break;
            }
            let strip = (line.len() - content.len()).min(indent);
            code.push_str(&line[strip..]);
            code.push('\n');
        }
        if info.split([',', ' ']).next() == Some(language) {
            blocks.push(code);
        }
    }
    blocks
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
