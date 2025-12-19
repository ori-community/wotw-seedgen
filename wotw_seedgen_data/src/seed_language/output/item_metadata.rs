use crate::{
    seed_language::output::{
        CommandInteger, CommandString, CommandVoid, ContainedWrites, StringOrPlaceholder,
    },
    CommonUberIdentifier, Icon, MapIcon, Skill,
};
use rand::{distributions::Uniform, prelude::Distribution, seq::SliceRandom};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemMetadata(pub(crate) FxHashMap<CommandVoid, ItemMetadataEntry>);
impl ItemMetadata {
    /// Look up metadata for `command`
    pub fn get<'command, 'entry>(
        &'entry self,
        command: &'command CommandVoid,
    ) -> ItemMetadataRef<'command, 'entry> {
        ItemMetadataRef {
            command,
            entry: self.0.get(command),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemMetadataRef<'command, 'entry> {
    command: &'command CommandVoid,
    entry: Option<&'entry ItemMetadataEntry>,
}

impl ItemMetadataRef<'_, '_> {
    /// Generic name used when sending the item to another world and in the spoiler.
    pub fn name(&self) -> Option<StringOrPlaceholder> {
        self.entry.and_then(|entry| entry.name.clone())
    }

    /// Force some kind of name for the item.
    ///
    /// If nothing is given by [`Self::name`], tries to scan the item for messages
    /// or even uses an approximate seedlang representation of it.
    pub fn force_name(&self) -> CommandString {
        self.name()
            .map(CommandString::from)
            .or_else(|| self.command.find_message().cloned())
            .unwrap_or_else(|| self.command.to_string().into())
    }

    /// Base price used when placed in a shop
    pub fn shop_price(&self) -> Option<CommandInteger> {
        self.entry.and_then(|entry| entry.shop_price.clone())
    }

    /// Force a shop price for the item.
    ///
    /// If nothing is given by [`Self::shop_price`], tries to estimate the item's
    /// value based on its contents and adds random noise to the result.
    pub fn force_shop_price(
        &self,
        price_distribution: &Uniform<f32>,
        rng: &mut Pcg64Mcg,
    ) -> CommandInteger {
        self.shop_price().unwrap_or_else(|| {
            let mut price = self
                .command
                .contained_common_write_identifiers()
                .map(CommonUberIdentifier::shop_price)
                .sum::<f32>();

            if price == 0. {
                price = 200.
            }
            if price != CommonUberIdentifier::Skill(Skill::Blaze).shop_price() {
                price *= price_distribution.sample(rng);
            }

            (price.round() as i32).into()
        })
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
            .unwrap_or_else(|| (*SHOP_DESCRIPTIONS.choose(rng).unwrap()).into())
    }

    /// Icon used when placed in a shop
    pub fn icon(&self) -> Option<Icon> {
        self.entry.and_then(|entry| entry.icon.clone())
    }

    /// Force an icon out of the item.
    ///
    /// If nothing is given by [`Self::icon`], tries to assign an icon based
    /// on the item's contents. May return `None` for unrecognized items.
    pub fn force_icon(&self) -> Option<Icon> {
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

    /// Force a map icon out of the item.
    ///
    /// If nothing is given by [`Self::map_icon`], tries to assign a map icon based
    /// on the item's contents, or returns [`MapIcon::default`]
    pub fn force_map_icon(&self) -> MapIcon {
        self.map_icon()
            .or_else(|| {
                self.command
                    .contained_common_write_identifiers()
                    .next()
                    .map(CommonUberIdentifier::map_icon)
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ItemMetadataEntry {
    pub name: Option<StringOrPlaceholder>, // TODO why not commandstring
    pub shop_price: Option<CommandInteger>,
    pub description: Option<CommandString>,
    pub icon: Option<Icon>,
    pub map_icon: Option<MapIcon>,
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
