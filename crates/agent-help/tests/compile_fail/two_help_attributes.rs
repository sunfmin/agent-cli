use agent_help::Help;

#[derive(Help)]
enum Refusal {
    #[help = "One."]
    #[help = "Two."]
    NameTaken,
}

fn main() {}
