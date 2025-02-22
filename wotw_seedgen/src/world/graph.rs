use wotw_seedgen_logic_language::output::Node;
use wotw_seedgen_seed_language::output::{
    CommandBoolean, CommandInteger, Comparator, Operation, Trigger,
};

// TODO should this be part of the nodes maybe instead of being constructed on demand?
fn node_condition_with_operator(node: &Node, operator: Comparator) -> Option<CommandBoolean> {
    node.uber_identifier()
        .map(|uber_identifier| match node.value() {
            None => CommandBoolean::FetchBoolean { uber_identifier },
            Some(value) => CommandBoolean::CompareInteger {
                operation: Box::new(Operation {
                    left: CommandInteger::FetchInteger { uber_identifier },
                    operator,
                    right: CommandInteger::Constant { value },
                }),
            },
        })
}
pub fn node_condition(node: &Node) -> Option<CommandBoolean> {
    node_condition_with_operator(node, Comparator::GreaterOrEqual)
}
pub fn node_condition_equals(node: &Node) -> Option<CommandBoolean> {
    node_condition_with_operator(node, Comparator::Equal)
}
pub fn node_trigger(node: &Node) -> Option<Trigger> {
    node_condition(node).map(Trigger::Condition)
}
