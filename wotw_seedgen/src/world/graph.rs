use wotw_seedgen_logic_language::output::Node;
use wotw_seedgen_seed_language::output::{
    CommandBoolean, CommandInteger, Comparator, EqualityComparator, Operation, Trigger,
};

pub fn node_condition(node: &Node) -> Option<CommandBoolean> {
    node.uber_identifier()
        .map(|uber_identifier| match node.value() {
            None => CommandBoolean::CompareBoolean {
                operation: Box::new(Operation {
                    left: CommandBoolean::FetchBoolean { uber_identifier },
                    operator: EqualityComparator::Equal,
                    right: CommandBoolean::Constant { value: true },
                }),
            },
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
