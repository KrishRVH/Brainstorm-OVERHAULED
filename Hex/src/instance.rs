use crate::item::{
    COMMON_JOKERS, ITEM_COUNT, Item, LEGENDARY_JOKERS, PACKS, PLANETS, Pack, RARE_JOKERS,
    SPECTRALS, ShopItem, TAGS, TAROTS, UNCOMMON_JOKERS, VOUCHERS,
};
use crate::rng::{LuaRandom, ante_to_string, fract, pseudohash_from, round13};
use crate::seed::Seed;

const SOURCE_SHOP: &str = "sho";
const SOURCE_ARCANA_PACK: &str = "ar1";
const SOURCE_OMEN_GLOBE: &str = "ar2";
const SOURCE_SPECTRAL_PACK: &str = "spe";
const SOURCE_BUFFOON_PACK: &str = "buf";
const SOURCE_SOUL: &str = "sou";

const RANDOM_JOKER_COMMON: &str = "Joker1";
const RANDOM_JOKER_UNCOMMON: &str = "Joker2";
const RANDOM_JOKER_RARE: &str = "Joker3";
const RANDOM_JOKER_LEGENDARY: &str = "Joker4";
const RANDOM_JOKER_RARITY: &str = "rarity";
const RANDOM_SHOP_PACK: &str = "shop_pack";
const RANDOM_TAROT: &str = "Tarot";
const RANDOM_SPECTRAL: &str = "Spectral";
const RANDOM_TAGS: &str = "Tag";
const RANDOM_CARD_TYPE: &str = "cdt";
const RANDOM_PLANET: &str = "Planet";
const RANDOM_VOUCHER: &str = "Voucher";
const RANDOM_SOUL: &str = "soul_";
const RANDOM_OMEN_GLOBE: &str = "omen_globe";

const KEY_CARD_TYPE_ANTE1: &str = "cdt1";
const KEY_SHOP_PACK_ANTE1: &str = "shop_pack1";
const KEY_TAG_ANTE1: &str = "Tag1";
const KEY_VOUCHER_ANTE1: &str = "Voucher1";
const KEY_JOKER_RARITY_SHOP_ANTE1: &str = "rarity1sho";
const KEY_JOKER_RARITY_BUFFOON_ANTE1: &str = "rarity1buf";
const KEY_JOKER_COMMON_SHOP_ANTE1: &str = "Joker1sho1";
const KEY_JOKER_COMMON_BUFFOON_ANTE1: &str = "Joker1buf1";
const KEY_JOKER_UNCOMMON_SHOP_ANTE1: &str = "Joker2sho1";
const KEY_JOKER_UNCOMMON_BUFFOON_ANTE1: &str = "Joker2buf1";
const KEY_JOKER_RARE_SHOP_ANTE1: &str = "Joker3sho1";
const KEY_JOKER_RARE_BUFFOON_ANTE1: &str = "Joker3buf1";

#[derive(Clone, Copy, Debug)]
pub(crate) struct ShopInstance {
    pub joker_rate: f64,
    pub tarot_rate: f64,
    pub planet_rate: f64,
    pub playing_card_rate: f64,
    pub spectral_rate: f64,
}

impl ShopInstance {
    fn total_rate(self) -> f64 {
        self.joker_rate
            + self.tarot_rate
            + self.planet_rate
            + self.playing_card_rate
            + self.spectral_rate
    }
}

#[derive(Clone, Debug)]
struct Cache {
    nodes: Vec<CacheNode>,
    active: usize,
    generated_first_pack: bool,
}

#[derive(Clone, Debug)]
struct CacheNode {
    key: String,
    value: f64,
}

impl Cache {
    fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(32),
            active: 0,
            generated_first_pack: false,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Instance {
    locked: [bool; ITEM_COUNT],
    pub seed: Seed,
    hashed_seed: f64,
    cache: Cache,
    deck: Item,
    vouchers: [bool; 32],
}

impl Instance {
    pub(crate) fn new(mut seed: Seed) -> Self {
        let hashed_seed = seed.pseudohash(0);
        Self {
            locked: [false; ITEM_COUNT],
            seed,
            hashed_seed,
            cache: Cache::new(),
            deck: Item::Red_Deck,
            vouchers: [false; 32],
        }
    }

    pub(crate) fn get_node(&mut self, id: &str) -> f64 {
        let position = self.cache.nodes[..self.cache.active]
            .iter()
            .position(|node| node.key == id);
        let node = if let Some(position) = position {
            &mut self.cache.nodes[position].value
        } else {
            let seed_hash = self.seed.pseudohash(id.len());
            let initial = pseudohash_from(id, seed_hash);
            let position = self.cache.active;
            self.cache.active += 1;
            if position == self.cache.nodes.len() {
                self.cache.nodes.push(CacheNode {
                    key: id.to_owned(),
                    value: initial,
                });
            } else {
                let node = &mut self.cache.nodes[position];
                node.key.clear();
                node.key.push_str(id);
                node.value = initial;
            }
            &mut self.cache.nodes[position].value
        };
        *node = round13(fract(*node * 1.72431234 + 2.134453429141));
        (*node + self.hashed_seed) / 2.0
    }

    pub(crate) fn random(&mut self, id: &str) -> f64 {
        let mut rng = LuaRandom::new(self.get_node(id));
        rng.random()
    }

    pub(crate) fn randchoice(&mut self, id: &str, items: &[Item]) -> Item {
        let mut rng = LuaRandom::new(self.get_node(id));
        let idx = rng.randint(0, items.len() as i32 - 1) as usize;
        let item = items[idx];
        if self.is_locked(item) || item == Item::RETRY {
            let mut resample = 2;
            loop {
                let resample_key = format!("{id}_resample{}", ante_to_string(resample));
                let mut rng = LuaRandom::new(self.get_node(&resample_key));
                let candidate = items[rng.randint(0, items.len() as i32 - 1) as usize];
                resample += 1;
                if (candidate != Item::RETRY && !self.is_locked(candidate)) || resample > 1000 {
                    return candidate;
                }
            }
        }
        item
    }

    fn randweightedchoice(&mut self, id: &str, items: &[crate::item::WeightedItem]) -> Item {
        let mut rng = LuaRandom::new(self.get_node(id));
        let poll = rng.random() * items[0].weight;
        let mut idx = 1_usize;
        let mut weight = 0.0;
        while weight < poll {
            weight += items[idx].weight;
            idx += 1;
        }
        items[idx - 1].item
    }

    pub(crate) fn lock(&mut self, item: Item) {
        if item.idx() < self.locked.len() {
            self.locked[item.idx()] = true;
        }
    }

    pub(crate) fn unlock(&mut self, item: Item) {
        if item.idx() < self.locked.len() {
            self.locked[item.idx()] = false;
        }
    }

    pub(crate) fn is_locked(&self, item: Item) -> bool {
        item.idx() < self.locked.len() && self.locked[item.idx()]
    }

    pub(crate) fn init_locks(&mut self, ante: i32, fresh_profile: bool, fresh_run: bool) {
        for pair in VOUCHERS.chunks_exact(2) {
            self.lock(pair[1]);
        }
        for item in [
            Item::Cavendish,
            Item::Steel_Joker,
            Item::Stone_Joker,
            Item::Lucky_Cat,
            Item::Golden_Ticket,
            Item::Glass_Joker,
        ] {
            self.lock(item);
        }
        if ante < 2 {
            self.lock_many(&[
                Item::The_Mouth,
                Item::The_Fish,
                Item::The_Wall,
                Item::The_House,
                Item::The_Mark,
                Item::The_Wheel,
                Item::The_Arm,
                Item::The_Water,
                Item::The_Needle,
                Item::The_Flint,
                Item::Negative_Tag,
                Item::Standard_Tag,
                Item::Meteor_Tag,
                Item::Buffoon_Tag,
                Item::Handy_Tag,
                Item::Garbage_Tag,
                Item::Ethereal_Tag,
                Item::Top_up_Tag,
                Item::Orbital_Tag,
            ]);
        }
        if ante < 3 {
            self.lock_many(&[Item::The_Tooth, Item::The_Eye]);
        }
        if ante < 4 {
            self.lock(Item::The_Plant);
        }
        if ante < 5 {
            self.lock(Item::The_Serpent);
        }
        if ante < 6 {
            self.lock(Item::The_Ox);
        }
        if fresh_profile {
            self.lock_many(&[
                Item::Negative_Tag,
                Item::Foil_Tag,
                Item::Holographic_Tag,
                Item::Polychrome_Tag,
                Item::Rare_Tag,
                Item::Golden_Ticket,
                Item::Mr_Bones,
                Item::Acrobat,
                Item::Sock_and_Buskin,
                Item::Swashbuckler,
                Item::Troubadour,
                Item::Certificate,
                Item::Smeared_Joker,
                Item::Throwback,
                Item::Hanging_Chad,
                Item::Rough_Gem,
                Item::Bloodstone,
                Item::Arrowhead,
                Item::Onyx_Agate,
                Item::Glass_Joker,
                Item::Showman,
                Item::Flower_Pot,
                Item::Blueprint,
                Item::Wee_Joker,
                Item::Merry_Andy,
                Item::Oops_All_6s,
                Item::The_Idol,
                Item::Seeing_Double,
                Item::Matador,
                Item::Hit_the_Road,
                Item::The_Duo,
                Item::The_Trio,
                Item::The_Family,
                Item::The_Order,
                Item::The_Tribe,
                Item::Stuntman,
                Item::Invisible_Joker,
                Item::Brainstorm,
                Item::Satellite,
                Item::Shoot_the_Moon,
                Item::Drivers_License,
                Item::Cartomancer,
                Item::Astronomer,
                Item::Burnt_Joker,
                Item::Bootstraps,
                Item::Overstock_Plus,
                Item::Liquidation,
                Item::Glow_Up,
                Item::Reroll_Glut,
                Item::Omen_Globe,
                Item::Observatory,
                Item::Nacho_Tong,
                Item::Recyclomancy,
                Item::Tarot_Tycoon,
                Item::Planet_Tycoon,
                Item::Money_Tree,
                Item::Antimatter,
                Item::Illusion,
                Item::Petroglyph,
                Item::Retcon,
                Item::Palette,
            ]);
        }
        if fresh_run {
            self.lock_many(&[
                Item::Planet_X,
                Item::Ceres,
                Item::Eris,
                Item::Five_of_a_Kind,
                Item::Flush_House,
                Item::Flush_Five,
                Item::Stone_Joker,
                Item::Steel_Joker,
                Item::Glass_Joker,
                Item::Golden_Ticket,
                Item::Lucky_Cat,
                Item::Cavendish,
                Item::Overstock_Plus,
                Item::Liquidation,
                Item::Glow_Up,
                Item::Reroll_Glut,
                Item::Omen_Globe,
                Item::Observatory,
                Item::Nacho_Tong,
                Item::Recyclomancy,
                Item::Tarot_Tycoon,
                Item::Planet_Tycoon,
                Item::Money_Tree,
                Item::Antimatter,
                Item::Illusion,
                Item::Petroglyph,
                Item::Retcon,
                Item::Palette,
            ]);
        }
    }

    fn lock_many(&mut self, items: &[Item]) {
        for item in items {
            self.lock(*item);
        }
    }

    pub(crate) fn next_tarot(&mut self, source: &str, ante: i32, soulable: bool) -> Item {
        let ante_str = ante_to_string(ante);
        if soulable
            && !self.is_locked(Item::The_Soul)
            && self.random(&format!("{RANDOM_SOUL}{RANDOM_TAROT}{ante_str}")) > 0.997
        {
            return Item::The_Soul;
        }
        self.randchoice(&format!("{RANDOM_TAROT}{source}{ante_str}"), &TAROTS)
    }

    pub(crate) fn next_planet(&mut self, source: &str, ante: i32, soulable: bool) -> Item {
        let ante_str = ante_to_string(ante);
        if soulable
            && !self.is_locked(Item::Black_Hole)
            && self.random(&format!("{RANDOM_SOUL}{RANDOM_PLANET}{ante_str}")) > 0.997
        {
            return Item::Black_Hole;
        }
        self.randchoice(&format!("{RANDOM_PLANET}{source}{ante_str}"), &PLANETS)
    }

    pub(crate) fn next_spectral(&mut self, source: &str, ante: i32, soulable: bool) -> Item {
        let ante_str = ante_to_string(ante);
        if soulable {
            let mut forced = Item::RETRY;
            if !self.is_locked(Item::The_Soul)
                && self.random(&format!("{RANDOM_SOUL}{RANDOM_SPECTRAL}{ante_str}")) > 0.997
            {
                forced = Item::The_Soul;
            }
            if !self.is_locked(Item::Black_Hole)
                && self.random(&format!("{RANDOM_SOUL}{RANDOM_SPECTRAL}{ante_str}")) > 0.997
            {
                forced = Item::Black_Hole;
            }
            if forced != Item::RETRY {
                return forced;
            }
        }
        self.randchoice(&format!("{RANDOM_SPECTRAL}{source}{ante_str}"), &SPECTRALS)
    }

    pub(crate) fn next_joker(&mut self, source: &str, ante: i32) -> Item {
        let ante_str = ante_to_string(ante);
        let rarity = if source == SOURCE_SOUL {
            Item::Legendary
        } else {
            let poll = if let Some(key) = joker_rarity_key(source, ante) {
                self.random(key)
            } else {
                self.random(&format!("{RANDOM_JOKER_RARITY}{ante_str}{source}"))
            };
            if poll > 0.95 {
                Item::Rare
            } else if poll > 0.7 {
                Item::Uncommon
            } else {
                Item::Common
            }
        };

        match rarity {
            Item::Legendary => self.randchoice(RANDOM_JOKER_LEGENDARY, &LEGENDARY_JOKERS),
            Item::Rare => {
                self.randchoice_joker_pool(RANDOM_JOKER_RARE, source, ante, &ante_str, &RARE_JOKERS)
            },
            Item::Uncommon => self.randchoice_joker_pool(
                RANDOM_JOKER_UNCOMMON,
                source,
                ante,
                &ante_str,
                &UNCOMMON_JOKERS,
            ),
            _ => self.randchoice_joker_pool(
                RANDOM_JOKER_COMMON,
                source,
                ante,
                &ante_str,
                &COMMON_JOKERS,
            ),
        }
    }

    fn randchoice_joker_pool(
        &mut self,
        prefix: &str,
        source: &str,
        ante: i32,
        ante_str: &str,
        items: &[Item],
    ) -> Item {
        if let Some(key) = joker_pool_key(prefix, source, ante) {
            self.randchoice(key, items)
        } else {
            self.randchoice(&format!("{prefix}{source}{ante_str}"), items)
        }
    }

    pub(crate) fn shop_instance(&self) -> ShopInstance {
        let mut tarot_rate = 4.0;
        let mut planet_rate = 4.0;
        let mut playing_card_rate = 0.0;
        let mut spectral_rate = 0.0;
        if self.deck == Item::Ghost_Deck {
            spectral_rate = 2.0;
        }
        if self.is_voucher_active(Item::Tarot_Tycoon) {
            tarot_rate = 32.0;
        } else if self.is_voucher_active(Item::Tarot_Merchant) {
            tarot_rate = 9.6;
        }
        if self.is_voucher_active(Item::Planet_Tycoon) {
            planet_rate = 32.0;
        } else if self.is_voucher_active(Item::Planet_Merchant) {
            planet_rate = 9.6;
        }
        if self.is_voucher_active(Item::Magic_Trick) {
            playing_card_rate = 4.0;
        }
        ShopInstance {
            joker_rate: 20.0,
            tarot_rate,
            planet_rate,
            playing_card_rate,
            spectral_rate,
        }
    }

    pub(crate) fn next_shop_item(&mut self, ante: i32) -> ShopItem {
        let ante_str = ante_to_string(ante);
        let shop = self.shop_instance();
        let cdt_poll = if ante == 1 {
            self.random(KEY_CARD_TYPE_ANTE1)
        } else {
            self.random(&format!("{RANDOM_CARD_TYPE}{ante_str}"))
        } * shop.total_rate();
        let item_type = shop_item_type(shop, cdt_poll);
        match item_type {
            Item::T_Joker => {
                let joker = self.next_joker(SOURCE_SHOP, ante);
                ShopItem {
                    item_type,
                    item: joker,
                }
            },
            Item::T_Tarot => ShopItem {
                item_type,
                item: self.next_tarot(SOURCE_SHOP, ante, false),
            },
            Item::T_Planet => ShopItem {
                item_type,
                item: self.next_planet(SOURCE_SHOP, ante, false),
            },
            Item::T_Spectral => ShopItem {
                item_type,
                item: self.next_spectral(SOURCE_SHOP, ante, false),
            },
            _ => ShopItem::default(),
        }
    }

    pub(crate) fn next_pack(&mut self, ante: i32) -> Item {
        if ante <= 2 && !self.cache.generated_first_pack {
            self.cache.generated_first_pack = true;
            return Item::Buffoon_Pack;
        }
        if ante == 1 {
            self.randweightedchoice(KEY_SHOP_PACK_ANTE1, &PACKS)
        } else {
            let ante_str = ante_to_string(ante);
            self.randweightedchoice(&format!("{RANDOM_SHOP_PACK}{ante_str}"), &PACKS)
        }
    }

    pub(crate) fn next_arcana_pack(&mut self, size: usize, ante: i32) -> Vec<Item> {
        let mut pack = Vec::with_capacity(size);
        for _ in 0..size {
            let item = if self.is_voucher_active(Item::Omen_Globe)
                && self.random(RANDOM_OMEN_GLOBE) > 0.8
            {
                self.next_spectral(SOURCE_OMEN_GLOBE, ante, true)
            } else {
                self.next_tarot(SOURCE_ARCANA_PACK, ante, true)
            };
            self.lock(item);
            pack.push(item);
        }
        for item in &pack {
            self.unlock(*item);
        }
        pack
    }

    pub(crate) fn next_spectral_pack(&mut self, size: usize, ante: i32) -> Vec<Item> {
        let mut pack = Vec::with_capacity(size);
        for _ in 0..size {
            let item = self.next_spectral(SOURCE_SPECTRAL_PACK, ante, true);
            self.lock(item);
            pack.push(item);
        }
        for item in &pack {
            self.unlock(*item);
        }
        pack
    }

    pub(crate) fn next_buffoon_pack(&mut self, size: usize, ante: i32) -> Vec<Item> {
        let mut pack = Vec::with_capacity(size);
        for _ in 0..size {
            let joker = self.next_joker(SOURCE_BUFFOON_PACK, ante);
            self.lock(joker);
            pack.push(joker);
        }
        for joker in &pack {
            self.unlock(*joker);
        }
        pack
    }

    pub(crate) fn is_voucher_active(&self, voucher: Item) -> bool {
        let start = Item::Overstock.idx();
        let idx = voucher.idx().saturating_sub(start);
        idx < self.vouchers.len() && self.vouchers[idx]
    }

    pub(crate) fn activate_voucher(&mut self, voucher: Item) {
        let start = Item::Overstock.idx();
        let idx = voucher.idx().saturating_sub(start);
        if idx < self.vouchers.len() {
            self.vouchers[idx] = true;
        }
        self.lock(voucher);
        for pair in VOUCHERS.chunks_exact(2) {
            if pair[0] == voucher {
                self.unlock(pair[1]);
            }
        }
    }

    pub(crate) fn next_voucher(&mut self, ante: i32) -> Item {
        if ante == 1 {
            self.randchoice(KEY_VOUCHER_ANTE1, &VOUCHERS)
        } else {
            self.randchoice(
                &format!("{RANDOM_VOUCHER}{}", ante_to_string(ante)),
                &VOUCHERS,
            )
        }
    }

    pub(crate) fn set_deck(&mut self, deck: Item) {
        self.deck = deck;
        if deck == Item::Magic_Deck {
            self.activate_voucher(Item::Crystal_Ball);
        }
        if deck == Item::Nebula_Deck {
            self.activate_voucher(Item::Telescope);
        }
        if deck == Item::Zodiac_Deck {
            self.activate_voucher(Item::Tarot_Merchant);
            self.activate_voucher(Item::Planet_Merchant);
            self.activate_voucher(Item::Overstock);
        }
    }

    pub(crate) fn next_tag(&mut self, ante: i32) -> Item {
        if ante == 1 {
            self.randchoice(KEY_TAG_ANTE1, &TAGS)
        } else {
            self.randchoice(&format!("{RANDOM_TAGS}{}", ante_to_string(ante)), &TAGS)
        }
    }
}

pub(crate) fn pack_info(pack: Item) -> Pack {
    const PACK_INFO: [Pack; 15] = [
        Pack {
            size: 3,
            choices: 1,
        },
        Pack {
            size: 5,
            choices: 1,
        },
        Pack {
            size: 5,
            choices: 2,
        },
        Pack {
            size: 3,
            choices: 1,
        },
        Pack {
            size: 5,
            choices: 1,
        },
        Pack {
            size: 5,
            choices: 2,
        },
        Pack {
            size: 3,
            choices: 1,
        },
        Pack {
            size: 5,
            choices: 1,
        },
        Pack {
            size: 5,
            choices: 2,
        },
        Pack {
            size: 2,
            choices: 1,
        },
        Pack {
            size: 4,
            choices: 1,
        },
        Pack {
            size: 4,
            choices: 2,
        },
        Pack {
            size: 2,
            choices: 1,
        },
        Pack {
            size: 4,
            choices: 1,
        },
        Pack {
            size: 4,
            choices: 2,
        },
    ];
    let idx = pack.idx() - Item::Arcana_Pack.idx();
    PACK_INFO[idx]
}

pub(crate) fn soul_yields_perkeo(inst: &mut Instance, ante: i32) -> bool {
    inst.next_joker(SOURCE_SOUL, ante) == Item::Perkeo
}

fn joker_rarity_key(source: &str, ante: i32) -> Option<&'static str> {
    if ante != 1 {
        return None;
    }
    match source {
        SOURCE_SHOP => Some(KEY_JOKER_RARITY_SHOP_ANTE1),
        SOURCE_BUFFOON_PACK => Some(KEY_JOKER_RARITY_BUFFOON_ANTE1),
        _ => None,
    }
}

fn joker_pool_key(prefix: &str, source: &str, ante: i32) -> Option<&'static str> {
    if ante != 1 {
        return None;
    }
    match (prefix, source) {
        (RANDOM_JOKER_COMMON, SOURCE_SHOP) => Some(KEY_JOKER_COMMON_SHOP_ANTE1),
        (RANDOM_JOKER_COMMON, SOURCE_BUFFOON_PACK) => Some(KEY_JOKER_COMMON_BUFFOON_ANTE1),
        (RANDOM_JOKER_UNCOMMON, SOURCE_SHOP) => Some(KEY_JOKER_UNCOMMON_SHOP_ANTE1),
        (RANDOM_JOKER_UNCOMMON, SOURCE_BUFFOON_PACK) => Some(KEY_JOKER_UNCOMMON_BUFFOON_ANTE1),
        (RANDOM_JOKER_RARE, SOURCE_SHOP) => Some(KEY_JOKER_RARE_SHOP_ANTE1),
        (RANDOM_JOKER_RARE, SOURCE_BUFFOON_PACK) => Some(KEY_JOKER_RARE_BUFFOON_ANTE1),
        _ => None,
    }
}

fn shop_item_type(shop: ShopInstance, mut cdt_poll: f64) -> Item {
    if cdt_poll < shop.joker_rate {
        return Item::T_Joker;
    }
    cdt_poll -= shop.joker_rate;
    if cdt_poll < shop.tarot_rate {
        return Item::T_Tarot;
    }
    cdt_poll -= shop.tarot_rate;
    if cdt_poll < shop.planet_rate {
        return Item::T_Planet;
    }
    cdt_poll -= shop.planet_rate;
    if cdt_poll < shop.playing_card_rate {
        return Item::T_Playing_Card;
    }
    Item::T_Spectral
}
