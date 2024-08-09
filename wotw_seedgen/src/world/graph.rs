use wotw_seedgen_logic_language::output::Node;
use wotw_seedgen_seed_language::output::{
    CommandBoolean, CommandInteger, Comparator, Operation, Trigger,
};

pub fn node_condition(node: &Node) -> Option<CommandBoolean> {
    node.uber_identifier()
        .map(|uber_identifier| match node.value() {
            None => CommandBoolean::FetchBoolean { uber_identifier },
            Some(value) => CommandBoolean::CompareInteger {
                operation: Box::new(Operation {
                    left: CommandInteger::FetchInteger { uber_identifier },
                    operator: Comparator::GreaterOrEqual,
                    right: CommandInteger::Constant {
                        value: value as i32,
                    },
                }),
            },
        })
}
pub fn node_trigger(node: &Node) -> Option<Trigger> {
    node_condition(node).map(Trigger::Condition)
}
