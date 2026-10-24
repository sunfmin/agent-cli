use agent_help::Help;

#[derive(Help)]
enum Refusal {
    /// The name is taken.
    NameTaken,
    OverLimit,
    /// The plan is gone.
    PlanGone,
    Expired,
}

fn main() {}
