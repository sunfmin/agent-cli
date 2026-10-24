use std::fmt;

/// A piece of the manual that belongs to no single command: how to sign in,
/// how to keep files on an object store. A tool answers `<binary> help <name>`
/// with its Markdown.
///
/// Derive it on an empty struct. Its name is `#[topic(name = "...")]`, in
/// kebab-case, and never `all`, which `<binary> help all` answers with the
/// whole manual; its Markdown is its doc comment, as written, most often a
/// file of its own, whose sections are `##` headings under the title the
/// manual writes (see [`Help`](struct@crate::Help)). It opens with a
/// sentence, the line the index shows of it; [`check`](crate::check) fails
/// on either. The crate names its binary
/// once, at its root, with [`binary!`](crate::binary):
///
/// ```
/// use agent_help::Topic;
///
/// agent_help::binary!("bk-server");
///
/// /// Run `bk-server login` once; it keeps the token in the keychain.
/// ///
/// /// - It asks nothing.
/// #[derive(Topic)]
/// #[topic(name = "sign-in")]
/// // or, its Markdown in a file of its own:
/// // #[doc = include_str!("sign-in.md")]
/// struct SignIn;
///
/// fn main() {
///     assert_eq!(SignIn::NAME, "sign-in");
///     assert_eq!(
///         SignIn::MARKDOWN,
///         "Run `bk-server login` once; it keeps the token in the keychain.\n\n- It asks nothing."
///     );
///     let error = format!("not signed in (run `{}`)", SignIn::REFERENCE);
///     assert_eq!(error, "not signed in (run `bk-server help sign-in`)");
/// }
/// ```
///
/// A message points at a topic by its [`REFERENCE`](Topic::REFERENCE), never
/// by its name written out: a topic renamed is renamed in every message, and
/// one removed fails to compile wherever a message still points at it.
///
/// A topic without a doc comment fails to compile, as does one whose name is
/// missing or not kebab-case:
///
/// ```compile_fail
/// agent_help::binary!("bk-server");
///
/// #[derive(agent_help::Topic)]
/// #[topic(name = "sign-in")]
/// struct SignIn;
///
/// fn main() {}
/// ```
pub trait Topic {
    /// What `<binary> help <name>` answers to: kebab-case, such as `sign-in`.
    const NAME: &'static str;
    /// The topic's doc comment, as written: each `///` line less the space
    /// after its slashes, a `#[doc = include_str!(...)]` file verbatim, the
    /// lines joined by a newline.
    const MARKDOWN: &'static str;
    /// The command that shows this topic, for a message to carry.
    const REFERENCE: TopicRef;
}

pub use agent_help_derive::Topic;

/// The command that shows a topic, `<binary> help <name>`, as its
/// [`Display`](fmt::Display) writes it. Only `#[derive(Topic)]` makes one, so
/// every reference names a topic that exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopicRef {
    binary: &'static str,
    topic: &'static str,
}

impl TopicRef {
    #[doc(hidden)]
    pub const fn __new(binary: &'static str, topic: &'static str) -> Self {
        Self { binary, topic }
    }

    /// The binary, as [`binary!`](crate::binary) names it.
    pub const fn binary(&self) -> &'static str {
        self.binary
    }

    /// The topic's [`NAME`](Topic::NAME).
    pub const fn topic(&self) -> &'static str {
        self.topic
    }
}

impl fmt::Display for TopicRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} help {}", self.binary, self.topic)
    }
}

/// Names the tool's binary, once, for every topic's
/// [`REFERENCE`](Topic::REFERENCE). Call it at the root of the crate that
/// derives its topics, before or after them; a crate deriving a topic without
/// it fails to compile, not finding `__AGENT_HELP_BINARY` in its root.
///
/// ```
/// agent_help::binary!("bk-server");
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! binary {
    ($name:expr) => {
        #[doc(hidden)]
        pub(crate) const __AGENT_HELP_BINARY: &str = $name;
    };
}
