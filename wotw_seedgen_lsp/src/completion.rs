use std::{ops::Range, sync::LazyLock};

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails};
use wotw_seedgen_data::{
    parse::{
        Delimited, Identifier, Once, Punctuated, Recoverable, SpanEnd, SpanStart, Spanned,
        SpannedOption,
    },
    seed_language::{
        ast::{
            Action, ActionCondition, AddItemArgs, Annotation, AugmentFunArgs, ChangeItemPoolArgs,
            ClientEvent, Command, CommandArg, CommandIf, CommandRepeat, ConfigArgs, ConfigType,
            ConstantDiscriminants, Content, CountInZoneArgs, CountInZoneBinding, Event, Expression,
            ExpressionValue, FunctionCall, FunctionDefinition, ItemDataArgs,
            ItemDataDescriptionArgs, ItemDataIconArgs, ItemDataMapIconArgs, ItemDataNameArgs,
            ItemDataPriceArgs, ItemOnArgs, LetArgs, Literal, Operation, PreplaceArgs,
            RandomFloatArgs, RandomIntegerArgs, RandomNumberArgs, RandomPoolArgs, RemoveItemArgs,
            RemoveLocationArgs, SeparatedNonEmpty, SetConfigArgs, Snippet, Span, SpawnArgs,
            StateArgs, TagsArg, Trigger, TriggerBinding, UberIdentifier, UberIdentifierName,
            UberIdentifierNumeric, UberStateType, ZoneOfArgs,
        },
        compile::FunctionIdentifier,
        types::Type,
    },
    Alignment, CoordinateSystem, EquipSlot, Equipment, GromIcon, HorizontalAnchor, LupoIcon,
    MapIcon, OpherIcon, ScreenPosition, Shard, Skill, Teleporter, TuleyIcon, VariantArray,
    VariantNames, VerticalAnchor, WeaponUpgrade, WheelBind, WheelItemPosition, Zone,
};

use crate::cache::CacheValues;

pub trait Completion {
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>>;
}

trait CompletionInSpan: Span {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>>;

    fn span_checked_completion(
        &self,
        span: Range<usize>,
        index: usize,
        cache: &CacheValues,
    ) -> Option<Vec<CompletionItem>> {
        if span.contains(&index) {
            self.completion_in_span(index, cache)
        } else {
            None
        }
    }
}

impl<T> Completion for T
where
    T: CompletionInSpan,
{
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.span_checked_completion(self.span(), index, cache)
    }
}

trait ErrCompletion {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem>;
}

fn keyword_completion(keyword: &str) -> CompletionItem {
    CompletionItem {
        label: keyword.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        ..Default::default()
    }
}

impl<T> CompletionInSpan for Box<T>
where
    T: Completion + Span,
{
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        (**self).completion(index, cache)
    }
}

impl<T> ErrCompletion for Box<T>
where
    T: ErrCompletion,
{
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        T::err_completion(cache)
    }
}

impl<T> Completion for Option<T>
where
    T: Completion,
{
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.as_ref().and_then(|t| t.completion(index, cache))
    }
}

impl<T> Completion for SpannedOption<T>
where
    T: Completion,
{
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.as_option().and_then(|t| t.completion(index, cache))
    }
}

impl<T> Completion for Vec<T>
where
    T: Completion + ErrCompletion,
{
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.iter()
            .find_map(|item| item.completion(index, cache))
            .or_else(|| Some(T::err_completion(cache)))
    }
}

impl<T> ErrCompletion for Vec<T>
where
    T: ErrCompletion,
{
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        T::err_completion(cache)
    }
}

impl<Open, Content, Close> CompletionInSpan for Delimited<Open, Content, Close>
where
    Open: SpanStart,
    Content: Completion + ErrCompletion,
    Close: SpanEnd,
{
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.content.completion(index, cache)
    }
}

impl<Open, Content, Close> ErrCompletion for Delimited<Open, Content, Close> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl<Item, Punctuation> Completion for Punctuated<Item, Punctuation>
where
    Item: Completion,
{
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.iter().find_map(|item| item.completion(index, cache))
    }
}

impl<Item, Punctuation> ErrCompletion for Punctuated<Item, Punctuation>
where
    Item: ErrCompletion,
{
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Item::err_completion(cache)
    }
}

impl<Item, Separator> CompletionInSpan for SeparatedNonEmpty<Item, Separator>
where
    Item: Completion + Span,
{
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.iter().find_map(|item| item.completion(index, cache))
    }
}

impl<Item, Separator> ErrCompletion for SeparatedNonEmpty<Item, Separator>
where
    Item: ErrCompletion,
{
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Item::err_completion(cache)
    }
}

impl<V> CompletionInSpan for Once<V>
where
    V: Completion + Span,
{
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.0.completion(index, cache)
    }
}

impl<V> ErrCompletion for Once<V>
where
    V: ErrCompletion,
{
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        V::err_completion(cache)
    }
}

impl<V, R> CompletionInSpan for Recoverable<V, R>
where
    V: Completion + ErrCompletion + Span,
{
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match &self.value {
            SpannedOption::None(_) => Some(V::err_completion(cache)),
            SpannedOption::Some(value) => value.completion(index, cache),
        }
    }
}

impl<V, R> ErrCompletion for Recoverable<V, R>
where
    V: ErrCompletion,
{
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        V::err_completion(cache)
    }
}

impl<T> CompletionInSpan for Spanned<T>
where
    T: Completion,
{
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.data.completion(index, cache)
    }
}

impl<T> ErrCompletion for Spanned<T>
where
    T: ErrCompletion,
{
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        T::err_completion(cache)
    }
}

impl<T> Completion for CommandArg<T>
where
    T: Completion + ErrCompletion + Span,
{
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.value
            .as_option()
            .and_then(|(_, t)| t.completion(index, cache))
    }
}

impl Completion for Identifier<'_> {
    fn completion(&self, _index: usize, _cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        // TODO offer identifiers in scope
        None
    }
}

impl CompletionInSpan for UberIdentifier<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match self {
            UberIdentifier::Numeric(uber_identifier_numeric) => {
                uber_identifier_numeric.completion(index, cache)
            }
            UberIdentifier::Name(uber_identifier_name) => {
                uber_identifier_name.completion(index, cache)
            }
        }
    }
}

impl CompletionInSpan for UberIdentifierNumeric {
    fn completion_in_span(
        &self,
        _index: usize,
        cache: &CacheValues,
    ) -> Option<Vec<CompletionItem>> {
        cache
            .uber_identifier_numeric_member_completion
            .get(&self.group.data)
            .cloned()
    }
}

impl CompletionInSpan for UberIdentifierName<'_> {
    fn completion_in_span(
        &self,
        _index: usize,
        cache: &CacheValues,
    ) -> Option<Vec<CompletionItem>> {
        cache
            .uber_identifier_name_member_completion
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

fn enum_member_completions<T>(variants: &[T]) -> Vec<CompletionItem>
where
    T: ToString,
{
    variants
        .iter()
        .map(|variant| CompletionItem {
            label: variant.to_string(),
            ..Default::default()
        })
        .collect()
}

fn enum_member_completions_detailed<T>(variants: &[T], detail: &str) -> Vec<CompletionItem>
where
    T: ToString,
{
    variants
        .iter()
        .map(|variant| CompletionItem {
            label: variant.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(format!(" ({detail})")),
                ..Default::default()
            }),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            ..Default::default()
        })
        .collect()
}

fn enum_member_completions_full<T, F, D>(
    variants: &[T],
    detail: &str,
    mut description: F,
) -> Vec<CompletionItem>
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
                detail: Some(format!(" ({detail})")),
                description: Some(description(variant).to_string()),
            }),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            ..Default::default()
        })
        .collect()
}

static CLIENT_EVENT_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions_detailed(ClientEvent::VARIANTS, "ClientEvent"));

static SKILL_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions_full(Skill::VARIANTS, "Skill", |skill| *skill as u8));

static SHARD_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions_full(Shard::VARIANTS, "Shard", |shard| *shard as u8));

static TELEPORTER_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(Teleporter::VARIANTS, "Teleporter", |teleporter| {
        *teleporter as u8
    })
});

static WEAPON_UPGRADE_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(WeaponUpgrade::VARIANTS, "WeaponUpgrade", |weapon_upgrade| {
        *weapon_upgrade as u8
    })
});

static EQUIPMENT_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(Equipment::VARIANTS, "Equipment", |equipment| {
        *equipment as u8
    })
});

static ZONE_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions_full(Zone::VARIANTS, "Zone", |zone| *zone as u8));

static OPHER_ICON_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(OpherIcon::VARIANTS, "OpherIcon", |opher_icon| {
        *opher_icon as u8
    })
});

static LUPO_ICON_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(LupoIcon::VARIANTS, "LupoIcon", |lupo_icon| *lupo_icon as u8)
});

static GROM_ICON_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(GromIcon::VARIANTS, "GromIcon", |grom_icon| *grom_icon as u8)
});

static TULEY_ICON_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(TuleyIcon::VARIANTS, "TuleyIcon", |tuley_icon| {
        *tuley_icon as u8
    })
});

static MAP_ICON_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(MapIcon::VARIANTS, "MapIcon", |map_icon| *map_icon as u8)
});

static EQUIP_SLOT_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(EquipSlot::VARIANTS, "EquipSlot", |equip_slot| {
        *equip_slot as u8
    })
});

static WHEEL_ITEM_POSITION_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(
        WheelItemPosition::VARIANTS,
        "WheelItemPosition",
        |wheel_item_position| *wheel_item_position as u8,
    )
});

static WHELL_BIND_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(WheelBind::VARIANTS, "WheelBind", |wheel_bind| {
        *wheel_bind as u8
    })
});

static ALIGNMENT_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(Alignment::VARIANTS, "Alignment", |alignment| {
        *alignment as u8
    })
});

static HORIZONTAL_ANCHOR_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(
        HorizontalAnchor::VARIANTS,
        "HorizontalAnchor",
        |horizontal_anchor| *horizontal_anchor as u8,
    )
});

static VERTICAL_ANCHOR_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(
        VerticalAnchor::VARIANTS,
        "VerticalAnchor",
        |vertical_anchor| *vertical_anchor as u8,
    )
});

static SCREEN_POSITION_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(
        ScreenPosition::VARIANTS,
        "ScreenPosition",
        |screen_position| *screen_position as u8,
    )
});

static COORDINATE_SYSTEM_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    enum_member_completions_full(
        CoordinateSystem::VARIANTS,
        "CoordinateSystem",
        |coordinate_system| *coordinate_system as u8,
    )
});

static CONSTANT_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    ConstantDiscriminants::VARIANTS
        .iter()
        .copied()
        .flat_map(constant_member_completion)
        .collect()
});

fn constant_member_completion(kind: ConstantDiscriminants) -> Vec<CompletionItem> {
    match kind {
        ConstantDiscriminants::ClientEvent => CLIENT_EVENT_COMPLETION.clone(),
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

static ACTION_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    // Important: we use this "if" in first position as a marker in Trigger::completion_after_span_check
    let mut completion = vec![keyword_completion("if")];

    completion.append(&mut FUNCTION_COMPLETION.clone());

    completion
});

static FUNCTION_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| function_completion());

fn literal_completion(cache: &CacheValues) -> Vec<CompletionItem> {
    let mut completion = cache.uber_identifier_numeric_completion.clone();

    completion.append(&mut CONSTANT_COMPLETION.clone());

    completion
}

fn expression_completion(cache: &CacheValues) -> Vec<CompletionItem> {
    // Important: we use the "if" from ACTION_COMPLETION in first position as a marker in Trigger::completion_after_span_check
    let mut completion = ACTION_COMPLETION.clone();

    completion.append(&mut literal_completion(cache));

    completion.append(&mut cache.uber_identifier_name_completion.clone());

    completion
}

impl CompletionInSpan for Expression<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match self {
            Expression::Value(value) => value.completion(index, cache),
            Expression::Operation(operation) => operation.completion(index, cache),
        }
    }
}

impl ErrCompletion for Expression<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        expression_completion(cache)
    }
}

impl CompletionInSpan for Operation<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.left
            .completion(index, cache)
            .or_else(|| self.right.completion(index, cache))
    }
}

impl CompletionInSpan for ExpressionValue<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match self {
            ExpressionValue::Group(group) => group.completion(index, cache),
            ExpressionValue::Action(action) => action.completion(index, cache),
            ExpressionValue::Literal(literal) => literal.completion(index, cache),
            ExpressionValue::Identifier(identifier) => {
                // Important: we use the "if" from EXPRESSION_COMPLETION in first position as a marker in Trigger::completion_after_span_check
                let mut completion = expression_completion(cache);

                if let Some(mut identifier_completion) = identifier.completion(index, cache) {
                    completion.append(&mut identifier_completion);
                }

                Some(completion)
            }
        }
    }
}

impl CompletionInSpan for Action<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match self {
            Action::Condition(_, condition) => condition.completion(index, cache),
            Action::Function(function_call) => function_call.completion(index, cache),
            Action::Multi(multi) => multi.completion(index, cache),
        }
    }
}

impl ErrCompletion for Action<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        ACTION_COMPLETION.clone()
    }
}

impl CompletionInSpan for ActionCondition<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.condition
            .completion(index, cache)
            .or_else(|| self.action.completion(index, cache))
    }
}

impl CompletionInSpan for FunctionCall<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.parameters.completion(index, cache)
    }
}

impl Completion for Literal<'_> {
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match self {
            Literal::UberIdentifier(uber_identifier) => uber_identifier.completion(index, cache),
            Literal::Integer(_) => Some(cache.uber_identifier_numeric_completion.clone()),
            Literal::Constant(_) => Some(CONSTANT_COMPLETION.clone()),
            Literal::Boolean(_) | Literal::Float(_) | Literal::String(_) => None,
        }
    }
}

impl ErrCompletion for Literal<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        literal_completion(cache)
    }
}

impl Completion for Snippet<'_> {
    fn completion(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.contents
            .iter()
            .find_map(|content| content.completion(index, cache))
    }
}

impl ErrCompletion for Snippet<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        unreachable!() // Snippet parsing cannot fail
    }
}

static CONTENT_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions(&["on", "fun"]));

impl CompletionInSpan for Content<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match self {
            Content::Event(on, event) => {
                event.span_checked_completion((on, event).span(), index, cache)
            }
            Content::Function(fun, function_definition) => function_definition
                .span_checked_completion((fun, function_definition).span(), index, cache),
            Content::Command(s, command) => {
                command.span_checked_completion((s, command).span(), index, cache)
            }
            Content::Annotation(s, annotation) => {
                annotation.span_checked_completion((s, annotation).span(), index, cache)
            }
        }
    }
}

impl ErrCompletion for Content<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        CONTENT_COMPLETION.clone()
    }
}

static TRIGGER_NON_EXPRESSION_COMPLETION: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    let mut completion = vec![keyword_completion("change")];

    completion.append(&mut CLIENT_EVENT_COMPLETION.clone());

    completion
});

fn trigger_completion(cache: &CacheValues) -> Vec<CompletionItem> {
    let mut completion = TRIGGER_NON_EXPRESSION_COMPLETION.clone();

    completion.append(&mut expression_completion(cache));

    completion
}

impl CompletionInSpan for Event<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.trigger
            .completion(index, cache)
            .or_else(|| self.action.completion(index, cache))
    }
}

impl ErrCompletion for Event<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        trigger_completion(cache)
    }
}

impl CompletionInSpan for Trigger<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match self {
            Trigger::ClientEvent(_) => None,
            Trigger::Binding(change, binding) => {
                binding.span_checked_completion((change, binding).span(), index, cache)
            }
            Trigger::Condition(condition) => {
                let mut completion = condition.completion(index, cache).unwrap_or_default();

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
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        trigger_completion(cache)
    }
}

impl CompletionInSpan for TriggerBinding<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        match self {
            TriggerBinding::UberIdentifier(uber_identifier) => {
                uber_identifier.completion(index, cache)
            }
            TriggerBinding::Identifier(identifier) => {
                let mut completion = cache.uber_identifier_name_completion.clone();

                if let Some(mut identifier_completion) = identifier.completion(index, cache) {
                    completion.append(&mut identifier_completion);
                }

                Some(completion)
            }
        }
    }
}

impl ErrCompletion for TriggerBinding<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        // Failure means there was no identifier, so only numeric completions may be relevant
        cache.uber_identifier_numeric_completion.clone()
    }
}

impl CompletionInSpan for FunctionDefinition<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.actions.completion(index, cache)
    }
}

impl ErrCompletion for FunctionDefinition<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

static COMMAND_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions(Command::VARIANTS));

impl CompletionInSpan for Command<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        // TODO need more recoveries inside commands for useful completions
        match self {
            Command::Include(_, _) => None, // TODO completions for files available in the workspace and identifiers contained in imported snippets
            Command::IncludeIcon(_, _) | Command::BuiltinIcon(_, _) => None, // TODO identifier and icon path completions
            Command::AugmentFun(augment_fun, args) => {
                args.span_checked_completion((augment_fun, args).span(), index, cache)
            }
            Command::Export(_, _) => None, // TODO identifier completions
            Command::Spawn(spawn, args) => {
                args.span_checked_completion((spawn, args).span(), index, cache)
            }
            Command::Tags(tags, args) => {
                args.span_checked_completion((tags, args).span(), index, cache)
            }
            Command::Config(config, args) => {
                args.span_checked_completion((config, args).span(), index, cache)
            }
            Command::SetConfig(set_config, args) => {
                args.span_checked_completion((set_config, args).span(), index, cache)
            }
            Command::State(state, args) => {
                args.span_checked_completion((state, args).span(), index, cache)
            }
            Command::Timer(_, _) => None,
            Command::Let(r#let, args) => {
                args.span_checked_completion((r#let, args).span(), index, cache)
            }
            Command::If(r#if, args) => {
                args.span_checked_completion((r#if, args).span(), index, cache)
            }
            Command::Repeat(repeat, args) => {
                args.span_checked_completion((repeat, args).span(), index, cache)
            }
            Command::AddItem(add_item, args) => {
                args.span_checked_completion((add_item, args).span(), index, cache)
            }
            Command::RemoveItem(remove_item, args) => {
                args.span_checked_completion((remove_item, args).span(), index, cache)
            }
            Command::ItemData(item_data, args) => {
                args.span_checked_completion((item_data, args).span(), index, cache)
            }
            Command::ItemDataName(item_data_name, args) => {
                args.span_checked_completion((item_data_name, args).span(), index, cache)
            }
            Command::ItemDataPrice(item_data_price, args) => {
                args.span_checked_completion((item_data_price, args).span(), index, cache)
            }
            Command::ItemDataDescription(item_data_description, args) => {
                args.span_checked_completion((item_data_description, args).span(), index, cache)
            }
            Command::ItemDataIcon(item_data_icon, args) => {
                args.span_checked_completion((item_data_icon, args).span(), index, cache)
            }
            Command::ItemDataMapIcon(item_data_map_icon, args) => {
                args.span_checked_completion((item_data_map_icon, args).span(), index, cache)
            }
            Command::RemoveLocation(remove_location, args) => {
                args.span_checked_completion((remove_location, args).span(), index, cache)
            }
            Command::SetLogicState(_, _) => None,
            Command::Preplace(preplace, args) => {
                args.span_checked_completion((preplace, args).span(), index, cache)
            }
            Command::ZoneOf(zone_of, args) => {
                args.span_checked_completion((zone_of, args).span(), index, cache)
            }
            Command::ItemOn(item_on, args) => {
                args.span_checked_completion((item_on, args).span(), index, cache)
            }
            Command::CountInZone(count_in_zone, args) => {
                args.span_checked_completion((count_in_zone, args).span(), index, cache)
            }
            Command::RandomInteger(random_integer, args) => {
                args.span_checked_completion((random_integer, args).span(), index, cache)
            }
            Command::RandomFloat(random_float, args) => {
                args.span_checked_completion((random_float, args).span(), index, cache)
            }
            Command::RandomPool(random_pool, args) => {
                args.span_checked_completion((random_pool, args).span(), index, cache)
            }
            Command::RandomFromPool(_, _) => None,
        }
    }
}

impl ErrCompletion for Command<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        // TODO typing ! doesn't give completions
        COMMAND_COMPLETION.clone()
    }
}

impl CompletionInSpan for AugmentFunArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.action.completion(index, cache)
    }
}

impl ErrCompletion for AugmentFunArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionInSpan for SpawnArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.x
            .completion(index, cache)
            .or_else(|| self.y.completion(index, cache))
    }
}

impl ErrCompletion for SpawnArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Expression::err_completion(cache) // TODO offer spawn identifiers from areas.wotw
    }
}

impl CompletionInSpan for TagsArg<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.0.completion(index, cache)
    }
}

impl ErrCompletion for TagsArg<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Expression::err_completion(cache)
    }
}

// TODO identifier completions in lots of places, just search for identifier in the ast...
impl CompletionInSpan for ConfigArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.ty
            .completion(index, cache)
            .or_else(|| self.default.completion(index, cache))
    }
}

impl ErrCompletion for ConfigArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

static CONFIG_TYPE_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions(ConfigType::VARIANTS));

impl Completion for ConfigType {
    fn completion(&self, _index: usize, _cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for ConfigType {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        CONFIG_TYPE_COMPLETION.clone()
    }
}

impl Completion for SetConfigArgs<'_> {
    fn completion(&self, _index: usize, _cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for SetConfigArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionInSpan for StateArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.ty.completion(index, cache)
    }
}

impl ErrCompletion for StateArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

static UBER_STATE_TYPE_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions(UberStateType::VARIANTS));

impl Completion for UberStateType {
    fn completion(&self, _index: usize, _cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for UberStateType {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        UBER_STATE_TYPE_COMPLETION.clone()
    }
}

impl CompletionInSpan for LetArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.value.completion(index, cache)
    }
}

impl ErrCompletion for LetArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionInSpan for CommandIf<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.condition
            .completion(index, cache)
            .or_else(|| self.contents.completion(index, cache))
    }
}

impl ErrCompletion for CommandIf<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Expression::err_completion(cache)
    }
}

impl CompletionInSpan for CommandRepeat<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.amount
            .completion(index, cache)
            .or_else(|| self.contents.completion(index, cache))
    }
}

impl ErrCompletion for CommandRepeat<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Expression::err_completion(cache)
    }
}

impl CompletionInSpan for AddItemArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.0.completion(index, cache)
    }
}

impl ErrCompletion for AddItemArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        ChangeItemPoolArgs::err_completion(cache)
    }
}

impl CompletionInSpan for RemoveItemArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.0.completion(index, cache)
    }
}

impl ErrCompletion for RemoveItemArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        ChangeItemPoolArgs::err_completion(cache)
    }
}

impl CompletionInSpan for ChangeItemPoolArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index, cache)
            .or_else(|| self.amount.completion(index, cache))
    }
}

impl ErrCompletion for ChangeItemPoolArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Action::err_completion(cache)
    }
}

impl CompletionInSpan for ItemDataArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index, cache)
            .or_else(|| self.name.completion(index, cache))
            .or_else(|| self.price.completion(index, cache))
            .or_else(|| self.description.completion(index, cache))
            .or_else(|| self.icon.completion(index, cache))
            .or_else(|| self.map_icon.completion(index, cache))
    }
}

impl ErrCompletion for ItemDataArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Action::err_completion(cache)
    }
}

impl CompletionInSpan for ItemDataNameArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index, cache)
            .or_else(|| self.name.completion(index, cache))
    }
}

impl ErrCompletion for ItemDataNameArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Action::err_completion(cache)
    }
}

impl CompletionInSpan for ItemDataPriceArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index, cache)
            .or_else(|| self.price.completion(index, cache))
    }
}

impl ErrCompletion for ItemDataPriceArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Action::err_completion(cache)
    }
}

impl CompletionInSpan for ItemDataDescriptionArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index, cache)
            .or_else(|| self.description.completion(index, cache))
    }
}

impl ErrCompletion for ItemDataDescriptionArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Action::err_completion(cache)
    }
}

impl CompletionInSpan for ItemDataIconArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index, cache)
            .or_else(|| self.icon.completion(index, cache))
    }
}

impl ErrCompletion for ItemDataIconArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Action::err_completion(cache)
    }
}

impl CompletionInSpan for ItemDataMapIconArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index, cache)
            .or_else(|| self.map_icon.completion(index, cache))
    }
}

impl ErrCompletion for ItemDataMapIconArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Action::err_completion(cache)
    }
}

impl CompletionInSpan for RemoveLocationArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.condition.completion(index, cache)
    }
}

impl ErrCompletion for RemoveLocationArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Expression::err_completion(cache)
    }
}

impl CompletionInSpan for PreplaceArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item
            .completion(index, cache)
            .or_else(|| self.zone.completion(index, cache))
    }
}

impl ErrCompletion for PreplaceArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        Action::err_completion(cache)
    }
}

impl CompletionInSpan for ZoneOfArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.item.completion(index, cache)
    }
}

impl ErrCompletion for ZoneOfArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionInSpan for ItemOnArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.trigger.completion(index, cache)
    }
}

impl ErrCompletion for ItemOnArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionInSpan for CountInZoneArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.zone_bindings
            .completion(index, cache)
            .or_else(|| self.items.completion(index, cache))
    }
}

impl ErrCompletion for CountInZoneArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionInSpan for CountInZoneBinding<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.zone.completion(index, cache)
    }
}

impl ErrCompletion for CountInZoneBinding<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionInSpan for RandomIntegerArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.0.completion(index, cache)
    }
}

impl ErrCompletion for RandomIntegerArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        RandomNumberArgs::err_completion(cache)
    }
}

impl CompletionInSpan for RandomFloatArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.0.completion(index, cache)
    }
}

impl ErrCompletion for RandomFloatArgs<'_> {
    fn err_completion(cache: &CacheValues) -> Vec<CompletionItem> {
        RandomNumberArgs::err_completion(cache)
    }
}

impl CompletionInSpan for RandomNumberArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.min
            .completion(index, cache)
            .or_else(|| self.max.completion(index, cache))
    }
}

impl ErrCompletion for RandomNumberArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

impl CompletionInSpan for RandomPoolArgs<'_> {
    fn completion_in_span(&self, index: usize, cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        self.ty
            .completion(index, cache)
            .or_else(|| self.values.completion(index, cache))
    }
}

impl ErrCompletion for RandomPoolArgs<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        vec![]
    }
}

static TYPE_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions(Type::VARIANTS));

impl Completion for Type {
    fn completion(&self, _index: usize, _cache: &CacheValues) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for Type {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        TYPE_COMPLETION.clone()
    }
}

static ANNOTATION_COMPLETION: LazyLock<Vec<CompletionItem>> =
    LazyLock::new(|| enum_member_completions(Annotation::VARIANTS));

impl CompletionInSpan for Annotation<'_> {
    fn completion_in_span(
        &self,
        _index: usize,
        _cache: &CacheValues,
    ) -> Option<Vec<CompletionItem>> {
        None
    }
}

impl ErrCompletion for Annotation<'_> {
    fn err_completion(_cache: &CacheValues) -> Vec<CompletionItem> {
        // TODO typing # doesn't give completions
        ANNOTATION_COMPLETION.clone()
    }
}
