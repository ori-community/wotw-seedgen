use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails};
use wotw_seedgen_assets::{
    data::{
        self, Alignment, CoordinateSystem, EquipSlot, Equipment, GromIcon, HorizontalAnchor,
        LupoIcon, MapIcon, OpherIcon, ScreenPosition, Shard, Skill, Teleporter, TuleyIcon,
        VariantArray, VerticalAnchor, WeaponUpgrade, WheelBind, WheelItemPosition, Zone,
    },
    UberStateAlias, UberStateDataEntry,
};
use wotw_seedgen_seed_language::{
    ast::{
        Action, ActionCondition, AddArgs, Annotation, ChangeItemPoolArgs, ClientEvent, Command,
        CommandArg, CommandIf, CommandRepeat, ConfigArgs, ConfigType, Constant, Content,
        CountInZoneArgs, CountInZoneBinding, Event, Expression, ExpressionValue, FunctionCall,
        FunctionDefinition, ItemDataArgs, ItemDataDescriptionArgs, ItemDataIconArgs,
        ItemDataNameArgs, ItemDataPriceArgs, ItemOnArgs, LetArgs, Literal, OnEventArgs, Operation,
        PreplaceArgs, RandomFloatArgs, RandomIntegerArgs, RandomNumberArgs, RandomPoolArgs,
        RemoveArgs, RemoveLocationArgs, Result, SeparatedNonEmpty, Snippet, Span, SpawnArgs,
        StateArgs, TagsArg, Trigger, TriggerBinding, UberIdentifier, UberIdentifierName,
        UberIdentifierNumeric, UberStateType, ZoneOfArgs,
    },
    compile::FunctionIdentifier,
    output::ConstantDiscriminants,
    parse::{
        Delimited, Error, ErrorKind, Identifier, NoTrailingInput, Once, Punctuated, Recoverable,
        SpanEnd, SpanStart, Spanned,
    },
    types::Type,
};
use wotw_seedgen_static_assets::UBER_STATE_DATA;

pub trait Completion {
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>>;
}

trait CompletionAfterSpanCheck: Span {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>>;
}

impl<T> Completion for T
where
    T: CompletionAfterSpanCheck,
{
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>> {
        if !self.span().contains(&index) {
            return None;
        }

        self.completion_after_span_check(index)
    }
}

trait ErrCompletion {
    fn err_completion(err: &Error) -> Vec<CompletionItem>;
}

fn keyword_completion(keyword: &str) -> CompletionItem {
    CompletionItem {
        label: keyword.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        ..Default::default()
    }
}

impl<T> CompletionAfterSpanCheck for Box<T>
where
    T: Completion + Span,
{
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        (**self).completion(index)
    }
}
impl<T> ErrCompletion for Box<T>
where
    T: ErrCompletion,
{
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        T::err_completion(err)
    }
}

impl<T> Completion for Result<T>
where
    T: Completion + ErrCompletion,
{
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            Ok(t) => t.completion(index),
            Err(err) => Some(T::err_completion(err)),
        }
    }
}

impl<T> Completion for Vec<T>
where
    T: Completion + ErrCompletion,
{
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.iter()
            .find_map(|item| item.completion(index))
            .or_else(|| Some(T::err_completion(&Error::custom(String::new(), 0..0))))
    }
}
impl<T> ErrCompletion for Vec<T>
where
    T: ErrCompletion,
{
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        T::err_completion(err)
    }
}

impl<Open, Content, Close> CompletionAfterSpanCheck for Delimited<Open, Content, Close>
where
    Open: SpanStart,
    Content: Completion + ErrCompletion,
    Close: SpanEnd,
{
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.content.completion(index)
    }
}

impl<Open, Content, Close> ErrCompletion for Delimited<Open, Content, Close> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl<Item, Punctuation> Completion for Punctuated<Item, Punctuation>
where
    Item: Completion,
{
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.iter().find_map(|item| item.completion(index))
    }
}

impl<Item, Punctuation> ErrCompletion for Punctuated<Item, Punctuation>
where
    Item: ErrCompletion,
{
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Item::err_completion(err)
    }
}

impl<Item, Separator> CompletionAfterSpanCheck for SeparatedNonEmpty<Item, Separator>
where
    Item: Completion + Span,
{
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.iter().find_map(|item| item.completion(index))
    }
}

impl<Item, Separator> ErrCompletion for SeparatedNonEmpty<Item, Separator>
where
    Item: ErrCompletion,
{
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Item::err_completion(err)
    }
}

impl<V> CompletionAfterSpanCheck for Once<V>
where
    V: Completion + Span,
{
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.0.completion(index)
    }
}
impl<V> ErrCompletion for Once<V>
where
    V: ErrCompletion,
{
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        V::err_completion(err)
    }
}

impl<T> Completion for NoTrailingInput<T>
where
    T: Completion + ErrCompletion,
{
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.parsed.completion(index)
    }
}

impl<V, R> CompletionAfterSpanCheck for Recoverable<V, R>
where
    V: Completion + ErrCompletion + Span,
{
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.result.completion(index)
    }
}
impl<V, R> ErrCompletion for Recoverable<V, R>
where
    V: ErrCompletion,
{
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        V::err_completion(err)
    }
}

impl<T> CompletionAfterSpanCheck for Spanned<T>
where
    T: Completion,
{
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.data.completion(index)
    }
}
impl<T> ErrCompletion for Spanned<T>
where
    T: ErrCompletion,
{
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        T::err_completion(err)
    }
}

impl<T> Completion for CommandArg<T>
where
    T: Completion + ErrCompletion + Span,
{
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.result
            .as_ref()
            .ok()
            .and_then(|(_, t)| t.completion(index))
    }
}

impl Completion for Identifier<'_> {
    fn completion(&self, _index: usize) -> Option<Vec<CompletionItem>> {
        // TODO offer identifiers in scope
        None
    }
}

fn uber_identifier_numeric_completion_item(
    id: data::UberIdentifier,
    data: &UberStateDataEntry,
) -> CompletionItem {
    CompletionItem {
        label: id.to_string(),
        label_details: Some(CompletionItemLabelDetails {
            description: Some(data.preferred_name().clone()),
            ..Default::default()
        }),
        kind: Some(CompletionItemKind::VALUE),
        ..Default::default()
    }
}

fn uber_identifier_name_completion_item(
    group: &str,
    member: &str,
    alias: &UberStateAlias,
    ambiguous: bool,
) -> CompletionItem {
    CompletionItem {
        label: format!("{group}.{member}"),
        label_details: Some(CompletionItemLabelDetails {
            description: Some(alias.to_string()),
            detail: ambiguous.then(|| "(ambiguous name)".to_string()),
        }),
        kind: Some(CompletionItemKind::VALUE),
        ..Default::default()
    }
}

lazy_static! {
    static ref UBER_IDENTIFIER_NUMERIC_COMPLETION: Vec<CompletionItem> =
        UBER_STATE_DATA
            .id_lookup
            .iter()
            .map(|(id, data)| uber_identifier_numeric_completion_item(*id, data))
            .collect()
    ;
    static ref UBER_IDENTIFIER_NUMERIC_MEMBER_COMPLETION: FxHashMap<i32, Vec<CompletionItem>> = {
        let mut group_map = FxHashMap::<i32, Vec<CompletionItem>>::default();

        for (id, data) in &UBER_STATE_DATA.id_lookup {
            group_map.entry(id.group).or_default().push(CompletionItem {
                insert_text: Some(id.member.to_string()),
                filter_text: Some(id.member.to_string()),
                ..uber_identifier_numeric_completion_item(*id, data)
            });
        }

        group_map
    };
    static ref UBER_IDENTIFIER_NAME_COMPLETION: Vec<CompletionItem> =
        UBER_STATE_DATA
            .name_lookup
            .iter()
            .flat_map(|(group, members)| {
                members.iter().flat_map(move |(member, aliases)| {
                    let ambiguous = aliases.len() > 1;

                    aliases.iter().map(move |alias| uber_identifier_name_completion_item(group, member, alias, ambiguous))
                })
            })
            .collect()
    ;
    static ref UBER_IDENTIFIER_NAME_MEMBER_COMPLETION: FxHashMap<String, Vec<CompletionItem>> = {
        UBER_STATE_DATA
            .name_lookup
            .iter()
            .map(|(group, members)| {
                (
                    group.clone(),
                        members
                            .iter()
                            .flat_map(|(member, aliases)| {
                                let ambiguous = aliases.len() > 1;

                                aliases.iter().map(move |alias| CompletionItem {
                                    insert_text: Some(member.clone()), // TODO edit in numbers on ambiguous names?
                                    filter_text: Some(member.clone()),
                                    ..uber_identifier_name_completion_item(group, member, alias, ambiguous)
                                })
                            })
                            .collect(),
                )
            })
            .collect()
    };
}

impl CompletionAfterSpanCheck for UberIdentifier<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            UberIdentifier::Numeric(uber_identifier_numeric) => {
                uber_identifier_numeric.completion(index)
            }
            UberIdentifier::Name(uber_identifier_name) => uber_identifier_name.completion(index),
        }
    }
}

impl CompletionAfterSpanCheck for UberIdentifierNumeric {
    fn completion_after_span_check(&self, _index: usize) -> Option<Vec<CompletionItem>> {
        UBER_IDENTIFIER_NUMERIC_MEMBER_COMPLETION
            .get(&self.group.data)
            .cloned()
    }
}

impl CompletionAfterSpanCheck for UberIdentifierName<'_> {
    fn completion_after_span_check(&self, _index: usize) -> Option<Vec<CompletionItem>> {
        UBER_IDENTIFIER_NAME_MEMBER_COMPLETION
            .get(self.group.data.0)
            .cloned()
    }
}

fn function_completion() -> Vec<CompletionItem> {
    // TODO abstract with enum_member_completion
    FunctionIdentifier::VARIANTS
        .iter()
        .map(|function_identifier| CompletionItem {
            label: function_identifier.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            ..Default::default()
        })
        .collect()
}

fn enum_member_completion_with_detail<T, F, D>(variants: &[T], mut detail: F) -> Vec<CompletionItem>
where
    T: ToString,
    F: FnMut(&T) -> D,
    D: ToString,
{
    variants
        .iter()
        .map(|variant| CompletionItem {
            label: variant.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                description: Some(detail(variant).to_string()),
                ..Default::default()
            }),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            ..Default::default()
        })
        .collect()
}

fn enum_member_completions<T>(variants: &[T]) -> impl Iterator<Item = CompletionItem> + use<'_, T>
where
    T: ToString,
{
    variants.iter().map(|variant| CompletionItem {
        label: variant.to_string(),
        ..Default::default()
    })
}

lazy_static! {
    static ref SKILL_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(Skill::VARIANTS, |skill| *skill as u8);
    static ref SHARD_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(Shard::VARIANTS, |shard| *shard as u8);
    static ref TELEPORTER_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(Teleporter::VARIANTS, |teleporter| *teleporter as u8);
    static ref WEAPON_UPGRADE_COMPLETION: Vec<CompletionItem> = enum_member_completion_with_detail(
        WeaponUpgrade::VARIANTS,
        |weapon_upgrade| *weapon_upgrade as u8
    );
    static ref EQUIPMENT_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(Equipment::VARIANTS, |equipment| *equipment as u8);
    static ref ZONE_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(Zone::VARIANTS, |zone| *zone as u8);
    static ref OPHER_ICON_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(OpherIcon::VARIANTS, |opher_icon| *opher_icon as u8);
    static ref LUPO_ICON_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(LupoIcon::VARIANTS, |lupo_icon| *lupo_icon as u8);
    static ref GROM_ICON_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(GromIcon::VARIANTS, |grom_icon| *grom_icon as u8);
    static ref TULEY_ICON_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(TuleyIcon::VARIANTS, |tuley_icon| *tuley_icon as u8);
    static ref MAP_ICON_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(MapIcon::VARIANTS, |map_icon| *map_icon as u8);
    static ref EQUIP_SLOT_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(EquipSlot::VARIANTS, |equip_slot| *equip_slot as u8);
    static ref WHEEL_ITEM_POSITION_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(WheelItemPosition::VARIANTS, |wheel_item_position| {
            *wheel_item_position as u8
        });
    static ref WHELL_BIND_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(WheelBind::VARIANTS, |wheel_bind| *wheel_bind as u8);
    static ref ALIGNMENT_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(Alignment::VARIANTS, |alignment| *alignment as u8);
    static ref HORIZONTAL_ANCHOR_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(HorizontalAnchor::VARIANTS, |horizontal_anchor| {
            *horizontal_anchor as u8
        });
    static ref VERTICAL_ANCHOR_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(VerticalAnchor::VARIANTS, |vertical_anchor| {
            *vertical_anchor as u8
        });
    static ref SCREEN_POSITION_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(ScreenPosition::VARIANTS, |screen_position| {
            *screen_position as u8
        });
    static ref COORDINATE_SYSTEM_COMPLETION: Vec<CompletionItem> =
        enum_member_completion_with_detail(CoordinateSystem::VARIANTS, |coordinate_system| {
            *coordinate_system as u8
        });
}

fn constant_member_completion(kind: ConstantDiscriminants) -> Vec<CompletionItem> {
    match kind {
        ConstantDiscriminants::Skill => SKILL_COMPLETION.clone(),
        ConstantDiscriminants::Shard => SHARD_COMPLETION.clone(),
        ConstantDiscriminants::Teleporter => TELEPORTER_COMPLETION.clone(),
        ConstantDiscriminants::WeaponUpgrade => WEAPON_UPGRADE_COMPLETION.clone(),
        ConstantDiscriminants::Equipment => EQUIPMENT_COMPLETION.clone(),
        ConstantDiscriminants::Zone => ZONE_COMPLETION.clone(),
        ConstantDiscriminants::OpherIcon => OPHER_ICON_COMPLETION.clone(),
        ConstantDiscriminants::LupoIcon => LUPO_ICON_COMPLETION.clone(),
        ConstantDiscriminants::GromIcon => GROM_ICON_COMPLETION.clone(),
        ConstantDiscriminants::TuleyIcon => TULEY_ICON_COMPLETION.clone(),
        ConstantDiscriminants::MapIcon => MAP_ICON_COMPLETION.clone(),
        ConstantDiscriminants::EquipSlot => EQUIP_SLOT_COMPLETION.clone(),
        ConstantDiscriminants::WheelItemPosition => WHEEL_ITEM_POSITION_COMPLETION.clone(),
        ConstantDiscriminants::WheelBind => WHELL_BIND_COMPLETION.clone(),
        ConstantDiscriminants::Alignment => ALIGNMENT_COMPLETION.clone(),
        ConstantDiscriminants::HorizontalAnchor => HORIZONTAL_ANCHOR_COMPLETION.clone(),
        ConstantDiscriminants::VerticalAnchor => VERTICAL_ANCHOR_COMPLETION.clone(),
        ConstantDiscriminants::ScreenPosition => SCREEN_POSITION_COMPLETION.clone(),
        ConstantDiscriminants::CoordinateSystem => COORDINATE_SYSTEM_COMPLETION.clone(),
    }
}

fn constant_completion() -> impl Iterator<Item = CompletionItem> {
    ConstantDiscriminants::VARIANTS.iter().flat_map(|kind| {
        let mut member_completion = constant_member_completion(*kind);
        member_completion
            .iter_mut()
            .for_each(|item| item.label = format!("{kind}::{member}", member = item.label));

        let kind_completion = CompletionItem {
            label: kind.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            ..Default::default()
        };

        member_completion.push(kind_completion);
        member_completion
    })
}

lazy_static! {
    static ref EXPRESSION_COMPLETION: Vec<CompletionItem> = {
        // Important: we use the "if" from ACTION_COMPLETION in first position as a marker in Trigger::completion_after_span_check
        let mut completion = ACTION_COMPLETION.clone();

        completion.append(&mut LITERAL_COMPLETION.clone());

        // TODO is this used elsewhere? If not, can save the clone
        completion.append(&mut UBER_IDENTIFIER_NAME_COMPLETION.clone());

        completion
    };

    static ref ACTION_COMPLETION: Vec<CompletionItem> = {
        // Important: we use this "if" in first position as a marker in Trigger::completion_after_span_check
        let mut completion = vec![keyword_completion("if")];

        completion.append(&mut FUNCTION_COMPLETION.clone());

        completion
    };

    static ref FUNCTION_COMPLETION: Vec<CompletionItem> = function_completion();

    static ref LITERAL_COMPLETION: Vec<CompletionItem> = {
        let mut completion = UBER_IDENTIFIER_NUMERIC_COMPLETION.clone();

        completion.extend(constant_completion());

        completion
    };
}

impl CompletionAfterSpanCheck for Expression<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            Expression::Value(value) => value.completion(index),
            Expression::Operation(operation) => operation.completion(index),
        }
    }
}

impl ErrCompletion for Expression<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        EXPRESSION_COMPLETION.clone()
    }
}

impl CompletionAfterSpanCheck for Operation<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.left
            .completion(index)
            .or_else(|| self.right.completion(index))
    }
}

impl CompletionAfterSpanCheck for ExpressionValue<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            ExpressionValue::Group(group) => group.completion(index),
            ExpressionValue::Action(action) => action.completion(index),
            ExpressionValue::Literal(literal) => literal.completion(index),
            ExpressionValue::Identifier(identifier) => {
                // Important: we use the "if" from EXPRESSION_COMPLETION in first position as a marker in Trigger::completion_after_span_check
                let mut completion = EXPRESSION_COMPLETION.clone();

                if let Some(mut identifier_completion) = identifier.completion(index) {
                    completion.append(&mut identifier_completion);
                }

                Some(completion)
            }
        }
    }
}

impl CompletionAfterSpanCheck for Action<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            Action::Condition(_, condition) => condition.completion(index),
            Action::Function(function_call) => function_call.completion(index),
            Action::Multi(multi) => multi.completion(index),
        }
    }
}

impl ErrCompletion for Action<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        ACTION_COMPLETION.clone()
    }
}

impl CompletionAfterSpanCheck for ActionCondition<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.condition
            .completion(index)
            .or_else(|| self.action.completion(index))
    }
}

impl CompletionAfterSpanCheck for FunctionCall<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.parameters.completion(index)
    }
}

impl Completion for Literal<'_> {
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            Literal::UberIdentifier(uber_identifier) => uber_identifier.completion(index),
            Literal::Integer(_) => Some(UBER_IDENTIFIER_NUMERIC_COMPLETION.clone()),
            Literal::Constant(constant) => constant.completion(index),
            Literal::Boolean(_) | Literal::Float(_) | Literal::String(_) => None,
        }
    }
}

impl ErrCompletion for Literal<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        LITERAL_COMPLETION.clone()
    }
}

impl Completion for Constant<'_> {
    fn completion(&self, _index: usize) -> Option<Vec<CompletionItem>> {
        let kind = self.kind.data.0.parse().ok()?;

        Some(constant_member_completion(kind))
    }
}

impl Completion for Snippet<'_> {
    fn completion(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.contents
            .iter()
            .find_map(|content| content.completion(index))
    }
}

impl ErrCompletion for Snippet<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        unreachable!() // Snippet parsing cannot fail
    }
}

impl CompletionAfterSpanCheck for Content<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            Content::Event(_, event) => event.completion(index),
            Content::Function(_, function_definition) => function_definition.completion(index),
            Content::Command(_, command) => command.completion(index),
            Content::Annotation(_, annotation) => annotation.completion(index),
        }
    }
}

/// Creates a list of [`CompletionItem`]s based on the [`Error`].
///
/// It will return a [`CompletionItem`] for every `ErrorKind::ExpectedToken(token)`
/// where `token` is surrounded by double quotes.
///
/// # Panics
///
/// Panics if `err.kind` does not match `ErrorKind::AllFailed(_)`.
fn completion_from_error(err: &Error) -> Vec<CompletionItem> {
    let ErrorKind::AllFailed(options) = &err.kind else {
        panic!("Malformed error passed to completion_from_error");
    };

    options
        .iter()
        .filter_map(|option| match option {
            ErrorKind::ExpectedToken(token) => token
                .strip_prefix('"')
                .and_then(|token| token.strip_suffix('"')),
            _ => None,
        })
        .map(|token| CompletionItem {
            label: token.to_string(),
            ..Default::default()
        })
        .collect()
}

impl ErrCompletion for Content<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        completion_from_error(err)
    }
}

lazy_static! {
    static ref TRIGGER_NON_EXPRESSION_COMPLETION: Vec<CompletionItem> = {
        let mut completion = vec![keyword_completion("change")];

        completion.extend(enum_member_completions(ClientEvent::VARIANTS));

        completion
    };
    static ref TRIGGER_COMPLETION: Vec<CompletionItem> = {
        let mut completion = TRIGGER_NON_EXPRESSION_COMPLETION.clone();

        completion.append(&mut EXPRESSION_COMPLETION.clone());

        completion
    };
}

impl CompletionAfterSpanCheck for Event<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.trigger
            .completion(index)
            .or_else(|| self.action.completion(index))
    }
}

impl ErrCompletion for Event<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        TRIGGER_COMPLETION.clone()
    }
}

impl CompletionAfterSpanCheck for Trigger<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            Trigger::ClientEvent(_) => None,
            Trigger::Binding(_, binding) => binding.completion(index),
            Trigger::Condition(condition) => {
                let mut completion = condition.completion(index).unwrap_or_default();

                // If we are returning identifier completions, add relevant non expression completions
                if completion.first().is_some_and(|item| item.label == "if") {
                    completion.append(&mut TRIGGER_NON_EXPRESSION_COMPLETION.clone());
                }

                Some(completion)
            }
        }
    }
}

impl ErrCompletion for Trigger<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        TRIGGER_COMPLETION.clone()
    }
}

impl CompletionAfterSpanCheck for TriggerBinding<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        match self {
            TriggerBinding::UberIdentifier(uber_identifier) => uber_identifier.completion(index),
            TriggerBinding::Identifier(identifier) => {
                let mut completion = UBER_IDENTIFIER_NAME_COMPLETION.clone();

                if let Some(mut identifier_completion) = identifier.completion(index) {
                    completion.append(&mut identifier_completion);
                }

                Some(completion)
            }
        }
    }
}

impl ErrCompletion for TriggerBinding<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        // Failure means there was no identifier, so only numeric completions may be relevant
        UBER_IDENTIFIER_NUMERIC_COMPLETION.clone()
    }
}

impl CompletionAfterSpanCheck for FunctionDefinition<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.actions.completion(index)
    }
}

impl ErrCompletion for FunctionDefinition<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionAfterSpanCheck for Command<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        // TODO need more recoveries inside commands for useful completions
        match self {
            Command::Include(_, _) => None, // TODO completions for files available in the workspace and identifiers contained in imported snippets
            Command::BundleIcon(_, _) | Command::BuiltinIcon(_, _) => None, // TODO identifier and icon path completions
            Command::Event(_, _) => None,
            Command::OnEvent(_, args) => args.completion(index),
            Command::Export(_, _) => None, // TODO identifier completions
            Command::Spawn(_, args) => args.completion(index),
            Command::Tags(_, args) => args.completion(index),
            Command::Config(_, args) => args.completion(index),
            Command::State(_, args) => args.completion(index),
            Command::Timer(_, _) => None,
            Command::Let(_, args) => args.completion(index),
            Command::If(_, args) => args.completion(index),
            Command::Repeat(_, args) => args.completion(index),
            Command::Add(_, args) => args.completion(index),
            Command::Remove(_, args) => args.completion(index),
            Command::ItemData(_, args) => args.completion(index),
            Command::ItemDataName(_, args) => args.completion(index),
            Command::ItemDataPrice(_, args) => args.completion(index),
            Command::ItemDataDescription(_, args) => args.completion(index),
            Command::ItemDataIcon(_, args) => args.completion(index),
            Command::RemoveLocation(_, args) => args.completion(index),
            Command::SetLogicState(_, _) => None,
            Command::Preplace(_, args) => args.completion(index),
            Command::ZoneOf(_, args) => args.completion(index),
            Command::ItemOn(_, args) => args.completion(index),
            Command::CountInZone(_, args) => args.completion(index),
            Command::RandomInteger(_, args) => args.completion(index),
            Command::RandomFloat(_, args) => args.completion(index),
            Command::RandomPool(_, args) => args.completion(index),
            Command::RandomFromPool(_, _) => None,
        }
    }
}

impl ErrCompletion for Command<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        // TODO typing ! doesn't give completions
        completion_from_error(err)
    }
}

impl CompletionAfterSpanCheck for OnEventArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.action.completion(index)
    }
}

impl ErrCompletion for OnEventArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionAfterSpanCheck for SpawnArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.x
            .completion(index)
            .or_else(|| self.y.completion(index))
    }
}

impl ErrCompletion for SpawnArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Expression::err_completion(err) // TODO offer spawn identifiers from areas.wotw
    }
}

impl CompletionAfterSpanCheck for TagsArg<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.0.completion(index)
    }
}

impl ErrCompletion for TagsArg<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Expression::err_completion(err)
    }
}

// TODO identifier completions in lots of places, just search for identifier in the ast...
impl CompletionAfterSpanCheck for ConfigArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.ty
            .completion(index)
            .or_else(|| self.default.completion(index))
    }
}

impl ErrCompletion for ConfigArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

lazy_static! {
    static ref CONFIG_TYPE_COMPLETION: Vec<CompletionItem> =
        enum_member_completions(ConfigType::VARIANTS).collect();
}

impl Completion for ConfigType {
    fn completion(&self, _index: usize) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for ConfigType {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        CONFIG_TYPE_COMPLETION.clone()
    }
}

impl CompletionAfterSpanCheck for StateArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.ty.completion(index)
    }
}

impl ErrCompletion for StateArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

lazy_static! {
    static ref UBER_STATE_TYPE_COMPLETION: Vec<CompletionItem> =
        enum_member_completions(UberStateType::VARIANTS).collect();
}

impl Completion for UberStateType {
    fn completion(&self, _index: usize) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for UberStateType {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        UBER_STATE_TYPE_COMPLETION.clone()
    }
}

impl CompletionAfterSpanCheck for LetArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.value.completion(index)
    }
}

impl ErrCompletion for LetArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionAfterSpanCheck for CommandIf<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.condition
            .completion(index)
            .or_else(|| self.contents.completion(index))
    }
}

impl ErrCompletion for CommandIf<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Expression::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for CommandRepeat<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.amount
            .completion(index)
            .or_else(|| self.contents.completion(index))
    }
}

impl ErrCompletion for CommandRepeat<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Expression::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for AddArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.0.completion(index)
    }
}

impl ErrCompletion for AddArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        ChangeItemPoolArgs::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for RemoveArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.0.completion(index)
    }
}

impl ErrCompletion for RemoveArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        ChangeItemPoolArgs::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for ChangeItemPoolArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index)
            .or_else(|| self.amount.completion(index))
    }
}

impl ErrCompletion for ChangeItemPoolArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Action::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for ItemDataArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index)
            .or_else(|| self.name.completion(index))
            .or_else(|| self.price.completion(index))
            .or_else(|| self.description.completion(index))
            .or_else(|| self.icon.completion(index))
            .or_else(|| self.map_icon.completion(index))
    }
}

impl ErrCompletion for ItemDataArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Action::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for ItemDataNameArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index)
            .or_else(|| self.name.completion(index))
    }
}

impl ErrCompletion for ItemDataNameArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Action::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for ItemDataPriceArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index)
            .or_else(|| self.price.completion(index))
    }
}

impl ErrCompletion for ItemDataPriceArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Action::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for ItemDataDescriptionArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index)
            .or_else(|| self.description.completion(index))
    }
}

impl ErrCompletion for ItemDataDescriptionArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Action::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for ItemDataIconArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index)
            .or_else(|| self.icon.completion(index))
    }
}

impl ErrCompletion for ItemDataIconArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Action::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for RemoveLocationArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.condition.completion(index)
    }
}

impl ErrCompletion for RemoveLocationArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Expression::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for PreplaceArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index)
            .or_else(|| self.zone.completion(index))
    }
}

impl ErrCompletion for PreplaceArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        Action::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for ZoneOfArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.item.completion(index)
    }
}

impl ErrCompletion for ZoneOfArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionAfterSpanCheck for ItemOnArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.trigger.completion(index)
    }
}

impl ErrCompletion for ItemOnArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionAfterSpanCheck for CountInZoneArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.zone_bindings
            .completion(index)
            .or_else(|| self.items.completion(index))
    }
}

impl ErrCompletion for CountInZoneArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionAfterSpanCheck for CountInZoneBinding<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.zone.completion(index)
    }
}

impl ErrCompletion for CountInZoneBinding<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionAfterSpanCheck for RandomIntegerArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.0.completion(index)
    }
}

impl ErrCompletion for RandomIntegerArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        RandomNumberArgs::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for RandomFloatArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.0.completion(index)
    }
}

impl ErrCompletion for RandomFloatArgs<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        RandomNumberArgs::err_completion(err)
    }
}

impl CompletionAfterSpanCheck for RandomNumberArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.min
            .completion(index)
            .or_else(|| self.max.completion(index))
    }
}

impl ErrCompletion for RandomNumberArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionAfterSpanCheck for RandomPoolArgs<'_> {
    fn completion_after_span_check(&self, index: usize) -> Option<Vec<CompletionItem>> {
        self.ty
            .completion(index)
            .or_else(|| self.values.completion(index))
    }
}

impl ErrCompletion for RandomPoolArgs<'_> {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        vec![]
    }
}

lazy_static! {
    static ref TYPE_COMPLETION: Vec<CompletionItem> =
        enum_member_completions(Type::VARIANTS).collect();
}

impl Completion for Type {
    fn completion(&self, _index: usize) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for Type {
    fn err_completion(_err: &Error) -> Vec<CompletionItem> {
        TYPE_COMPLETION.clone()
    }
}

impl CompletionAfterSpanCheck for Annotation<'_> {
    fn completion_after_span_check(&self, _index: usize) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for Annotation<'_> {
    fn err_completion(err: &Error) -> Vec<CompletionItem> {
        // TODO typing # doesn't give completions
        completion_from_error(err)
    }
}
