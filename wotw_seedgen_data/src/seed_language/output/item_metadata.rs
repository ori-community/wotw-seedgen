use crate::{
    seed_language::{
        output::{
            display::strip_invisible_characters, CommandInteger, CommandString, CommandVoid,
            CommandsOutput, ContainedWrites, StringOrPlaceholder,
        },
        simulate::{Simulate, Simulation},
    },
    CommonUberIdentifier, Icon, MapIcon,
};
use derivative::Derivative;
use log::warn;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;
use rustc_hash::{FxBuildHasher, FxHashMap};
use wotw_seedgen_log_capture::{LogCapture, NO_LOG_CAPTURE};

// TODO fewer string allocations?

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct ItemMetadata<'log> {
    pub(crate) inner: FxHashMap<CommandVoid, ItemMetadataEntry>,
    #[derivative(PartialEq = "ignore")]
    pub(crate) log_capture: &'log LogCapture,
}

impl<'log> ItemMetadata<'log> {
    pub const fn new() -> Self {
        Self {
            inner: FxHashMap::with_hasher(FxBuildHasher),
            log_capture: &NO_LOG_CAPTURE,
        }
    }

    /// Look up metadata for `command`
    pub fn get<'command, 'entry>(
        &'entry self,
        command: &'command CommandVoid,
    ) -> ItemMetadataRef<'command, 'entry, 'log> {
        ItemMetadataRef {
            command,
            entry: self.inner.get(command),
            log_capture: self.log_capture,
        }
    }
}

impl Default for ItemMetadata<'static> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ItemMetadataRef<'command, 'entry, 'log> {
    command: &'command CommandVoid,
    entry: Option<&'entry ItemMetadataEntry>,
    log_capture: &'log LogCapture,
}

impl ItemMetadataRef<'_, '_, '_> {
    /// Generic name used when sending the item to another world and in the spoiler.
    pub fn name(&self) -> Option<StringOrPlaceholder> {
        self.entry.and_then(|entry| entry.name.clone())
    }

    /// Try to force some kind of name for the item.
    ///
    /// If nothing is given by [`Self::name`], tries to scan the item for messages.
    pub fn try_force_name(&self) -> Option<CommandString> {
        self.name()
            .map(CommandString::from)
            .or_else(|| self.command.contained_messages().next().cloned())
    }

    /// Force some kind of name for the item.
    ///
    /// If nothing is given by [`Self::try_force_name`], returns a code representation.
    pub fn force_name(&self) -> CommandString {
        self.try_force_name().unwrap_or_else(|| {
            let code = self.command.to_string();
            warn!(logger: self.log_capture, "unable to find readable name for {code}");
            code.into()
        })
    }

    /// Force some kind of name for the item that can be used in a log.
    ///
    /// Similar to [`Self::force_name`], but simulates the result to get a `String`
    /// and removes characters that wouldn't be rendered in an in-game message
    pub fn log_name<S: Simulation>(&self, simulation: &mut S, output: &CommandsOutput) -> String {
        let name = self.force_name().simulate(simulation, output);

        strip_invisible_characters(&name)
    }

    /// Base price used when placed in a shop
    pub fn shop_price(&self) -> Option<CommandInteger> {
        self.entry.and_then(|entry| entry.shop_price.clone())
    }

    /// Try to force a shop price for the item.
    ///
    /// If nothing is given by [`Self::shop_price`], tries to estimate the item's
    /// value based on its contents.
    pub fn try_force_shop_price(&self) -> Option<CommandInteger> {
        self.shop_price().or_else(|| {
            let price = self
                .command
                .contained_common_write_identifiers()
                .map(CommonUberIdentifier::shop_price)
                .sum::<i32>();

            (price > 0).then(|| price.into())
        })
    }

    /// Force a shop price for the item.
    ///
    /// If nothing is given by [`Self::try_force_shop_price`], defaults to [`DEFAULT_SHOP_PRICE`].
    pub fn force_shop_price(&self) -> CommandInteger {
        self.shop_price()
            .unwrap_or_else(|| DEFAULT_SHOP_PRICE.into())
    }

    /// Description used when placed in a shop
    pub fn description(&self) -> Option<CommandString> {
        self.entry.and_then(|entry| entry.description.clone())
    }

    /// Force a description for the item.
    ///
    /// If nothing is given by [`Self::description`], returns a random description.
    pub fn force_description(&self, rng: &mut Pcg64Mcg) -> CommandString {
        self.description()
            .unwrap_or_else(|| random_shop_description(rng).into())
    }

    /// Icon used when placed in a shop
    pub fn icon(&self) -> Option<Icon> {
        self.entry.and_then(|entry| entry.icon.clone())
    }

    /// Try to force an icon out of the item.
    ///
    /// If nothing is given by [`Self::icon`], tries to assign an icon based
    /// on the item's contents. May return `None` for unrecognized items.
    pub fn try_force_icon(&self) -> Option<Icon> {
        self.icon().or_else(|| {
            self.command
                .contained_common_write_identifiers()
                .next()
                .and_then(CommonUberIdentifier::icon)
        })
    }

    /// Map Icon used in the spoiler map
    pub fn map_icon(&self) -> Option<MapIcon> {
        self.entry.and_then(|entry| entry.map_icon)
    }

    /// Try to force a map icon out of the item.
    ///
    /// If nothing is given by [`Self::map_icon`], tries to assign a map icon based
    /// on the item's contents.
    pub fn try_force_map_icon(&self) -> Option<MapIcon> {
        self.map_icon().or_else(|| {
            self.command
                .contained_common_write_identifiers()
                .next()
                .map(CommonUberIdentifier::map_icon)
        })
    }

    /// Force a map icon out of the item.
    ///
    /// If nothing is given by [`Self::try_force_map_icon`], returns [`MapIcon::default`]
    pub fn force_map_icon(&self) -> MapIcon {
        self.try_force_map_icon().unwrap_or_default()
    }
}

// TODO cache computed metadata
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ItemMetadataEntry {
    pub name: Option<StringOrPlaceholder>, // TODO why not commandstring
    pub shop_price: Option<CommandInteger>,
    pub description: Option<CommandString>,
    pub icon: Option<Icon>,
    pub map_icon: Option<MapIcon>,
}

pub(crate) fn random_shop_description(rng: &mut Pcg64Mcg) -> &str {
    SHOP_DESCRIPTIONS.choose(rng).unwrap()
}

const SHOP_DESCRIPTIONS: [&str; 38] = [
    "Nice, isn't it?",
    "Very shiny",
    "One of my favorites",
    "I've always loved these",
    "Popular among the Moki",
    "A crowd favorite",
    "Seems kind of useless",
    "I guess someone could use this?",
    "I found this nearby",
    "Traded for this from a Moki",
    "Grom said he's never\nseen one of these",
    "Grom loves these",
    "Tokk gave me this",
    "Lupo found this while\nexploring the Wellspring",
    "Lupo found this deep\nin Inkwater Marsh",
    "Lupo found this under\nthe big statue of Kwolok",
    "Lupo found this floating\nin Luma Pools",
    "It's dangerous to go alone",
    "It's fresh!",
    "Hot item!",
    "Found this in the Midnight Burrows",
    "Fresh out of Nibel!",
    "I have no idea where this came from",
    "Not really sure what this is for",
    "You can use this, right?",
    "Selling this one at a loss",
    "Caveat emptor!",
    "Heh",
    "Look...",
    "Don't worry about it",
    "I used to give out\ncoupons for these",
    "Take it, please",
    "I think Howl coughed\nthis thing up",
    "Found it in Shriek's um... leavings",
    "Don't forget to take a picture\nfor social media",
    "9/10 dentists recommend this",
    "This one's good luck",
    "Better than a bowl of Marshclam Soup",
];

pub(crate) const DEFAULT_SHOP_PRICE: i32 = 200;
