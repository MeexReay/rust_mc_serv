use craftflow_nbt::DynNBT;
use enum_index::EnumIndex;
use enum_index_derive::EnumIndex;
use rust_mc_proto::{DataReader, DataWriter, Packet};
use uuid::Uuid;

use crate::ServerError;

use super::{IdOr, IdSet, Property, component::TextComponent, sound::SoundEvent};

pub const SLOT_COMPONENT_LENGTH: u16 = 96;

#[derive(Clone)]
pub struct JukeboxSong {
	pub sound: IdOr<SoundEvent>,
	pub description: TextComponent,
	/// The duration the songs should play for, in seconds.
	pub duration: f32,
	/// The output strength given by a comparator. Between 0 and 15.
	pub output: i32,
}

#[derive(Clone)]
pub enum JukeboxPlayable {
	ByName(String),
	ById(IdOr<JukeboxSong>),
}

#[derive(Clone)]
pub enum ChickenVariant {
	ByName(String),
	ById(u32),
}

#[derive(Clone)]
pub struct PaintingVariant {
	pub width: i32,
	pub height: i32,
	/// The texture for the painting. The Notchian client uses the corresponding asset located at textures/painting.
	pub asset_id: String,
	pub title: Option<TextComponent>,
	pub author: Option<TextComponent>,
}

#[derive(Clone)]
pub enum ProvidesTrimMaterial {
	ByName(String),
	ById(IdOr<TrimMaterial>),
}

#[derive(Clone)]
pub enum BlockPredicatePropertyMatch {
	Exact(String),
	Range(String, String),
}

#[derive(Clone)]
pub struct BlockPredicateExactDataMatcher {
	/// ID of the data component as listed in the table of data component types above.
	pub component_index: usize,
	pub value: StructuredComponent,
}

#[derive(Clone)]
pub struct BlockPredicatePartialDataMatcher {
	/// 0: damage, 1: enchantments, 2: stored_enchantments, 3: potion_contents, 4: custom_data, 5: container, 6: bundle_contents, 7: firework_explosion, 8: fireworks, 9: writable_book_content, 10: written_book_content, 11: attribute_modifiers, 12: trim, 13: jukebox_playable.
	pub type_id: u8,
	/// https://minecraft.wiki/w/Data_component_predicate
	pub predicate: DynNBT,
}

#[derive(Clone)]
pub struct BlockPredicateProperty {
	pub name: String,
	pub matches: BlockPredicatePropertyMatch,
}

#[derive(Clone)]
pub struct BlockPredicate {
	pub blocks: Option<IdSet>,
	pub properties: Option<Vec<BlockPredicateProperty>>,
	pub nbt: Option<DynNBT>,
	pub exact_matchers: Vec<BlockPredicateExactDataMatcher>,
	pub partial_matchers: Vec<BlockPredicatePartialDataMatcher>,
}

#[derive(Clone)]
pub struct Instrument {
	pub sound: IdOr<SoundEvent>,
	pub sound_range: f32,
	pub range: f32,
	pub description: TextComponent,
}

#[derive(Clone)]
pub enum ConsumeEffect {
	/// Effects, Probability
	ApplyEffects(Vec<PotionEffect>, f32),
	RemoveEffects(IdSet),
	ClearAllEffects,
	/// Diameter
	TeleportRandomly(f32),
	PlaySound(SoundEvent),
}

#[derive(Clone)]
pub struct TrimMaterial {
	pub suffix: String,
	/// Key - Armor Material Type Identifier
	/// Value - Overriden Asset Name
	pub overrides: Vec<(String, String)>,
	pub description: TextComponent,
}

#[derive(Clone)]
pub struct TrimPattern {
	pub asset_name: String,
	pub template_item: i32,
	pub description: TextComponent,
	pub decal: bool,
}

#[derive(Clone)]
pub struct PotionEffectDetail {
	pub amplifier: i32,
	/// -1 for infinite.
	pub duration: i32,
	pub ambient: bool,
	pub show_particles: bool,
	pub show_icon: bool,
	pub hidden_effect: Option<Box<PotionEffectDetail>>,
}

#[derive(Clone)]
pub struct PotionEffect {
	pub type_id: u32,
	pub detail: PotionEffectDetail,
}

#[derive(Clone)]
pub struct FireworkExplosion {
	/// Can be one of the following:
	/// - 0 - Small ball
	/// - 1 - Large ball
	/// - 2 - Star
	/// - 3 - Creeper
	/// - 4 - Burst
	pub shape: u8,
	pub colors: Vec<i32>,
	pub fade_colors: Vec<i32>,
	pub has_trail: bool,
	pub has_twinkle: bool,
}

#[derive(Clone)]
pub struct HiveBee {
	pub entity_data: DynNBT,
	pub ticks_in_hive: i32,
	pub min_ticks_in_hive: i32,
}

#[derive(Clone)]
pub struct BannerLayer {
	pub pattern_type: i32,
	pub asset_id: Option<String>,
	pub translation_key: Option<String>,
	/// Can be one of the following:
	/// - 0 - White
	/// - 1 - Orange
	/// - 2 - Magenta
	/// - 3 - Light Blue
	/// - 4 - Yellow
	/// - 5 - Lime
	/// - 6 - Pink
	/// - 7 - Gray
	/// - 8 - Light Gray
	/// - 9 - Cyan
	/// - 10 - Purple
	/// - 11 - Blue
	/// - 12 - Brown
	/// - 13 - Green
	/// - 14 - Red
	/// - 15 - Black
	pub color: u8,
}

#[derive(Clone)]
pub struct AttributeModifier {
	pub attribute_id: u64,
	pub modifier_id: String,
	pub value: f64,
	/// The operation to be applied upon the value. Can be one of the following:
	/// - 0 - Add
	/// - 1 - Multiply base
	/// - 2 - Multiply total
	pub operation: u8,
	/// The item slot placement required for the modifier to have effect.
	/// Can be one of the following:
	/// - 0 - Any
	/// - 1 - Main hand
	/// - 2 - Off hand
	/// - 3 - Hand
	/// - 4 - Feet
	/// - 5 - Legs
	/// - 6 - Chest
	/// - 7 - Head
	/// - 8 - Armor
	/// - 9 - Body
	pub slot: u8,
}

#[derive(Clone)]
pub struct ToolRule {
	pub blocks: IdSet,
	pub has_speed: bool,
	pub speed: Option<f32>,
	pub has_correct_drop_for_blocks: bool,
	pub correct_drop_for_blocks: Option<bool>,
}

#[derive(Clone)]
pub struct DamageReduction {
	pub horizontal_blocking_angle: f32,
	pub damage_kind: Option<IdSet>,
	pub base: f32,
	pub factor: f32,
}

/// https://minecraft.wiki/w/Java_Edition_protocol/Slot_data#Structured_components
#[derive(Clone, EnumIndex)]
pub enum StructuredComponent {
	CustomData(DynNBT),
	/// 1 - 99
	MaxStackSize(i32),
	MaxDamage(i32),
	Damage(i32),
	Unbreakable,
	CustomName(TextComponent),
	ItemName(TextComponent),
	ItemModel(String),
	Lore(TextComponent),
	/// Can be one of the following:
	/// - 0 - Common (white)
	/// - 1 - Uncommon (yellow)
	/// - 2 - Rare (aqua)
	/// - 3 - Epic (pink)
	Rarity(u8),
	/// Key: The ID of the enchantment in the enchantment registry. \
	/// Value: The level of the enchantment.
	Enchantments(Vec<(u64, i32)>),
	CanPlaceOn(Vec<BlockPredicate>),
	CanBreak(Vec<BlockPredicate>),
	AttributeModifiers(Vec<AttributeModifier>),
	/// Floats, Flags, Strings, Colors
	CustomModelData(Vec<f32>, Vec<bool>, Vec<String>, Vec<i32>),
	/// Parameters:
	/// - Hide Tooltip
	/// - The IDs of data components in the minecraft:data_component_type registry to hide.
	TooltipDisplay(bool, Vec<u64>),
	/// Accumulated anvil usage cost.
	/// The client displays "Too Expensive" if the value is greater than 40 and
	/// the player is not in creative mode (more specifically,
	/// if they don't have the insta-build flag enabled).
	/// This behavior can be overridden by setting the level
	/// with the Set Container Property packet.
	RepairCost(i32),
	/// Marks the item as non-interactive on the creative inventory (the first 5 rows of items). \
	/// This is used internally by the client on the paper icon in the saved hot-bars tab.
	CreativeSlotLock,
	EnchantmentGlintOverride(bool),
	/// Marks the projectile as intangible (cannot be picked-up).
	IntangibleProjectile,
	/// Parameters:
	/// - Nutrition, Non-negative
	/// - How much saturation will be given after consuming the item.
	/// - Whether the item can always be eaten, even at full hunger.
	Food(u32, f32, bool),
	/// Animation: 0: none, 1: eat, 2: drink, 3: block, 4: bow, 5: spear, 6: crossbow, 7: spyglass, 8: toot_horn, 9: brush
	Consumable {
		consume_seconds: f32,
		animation: u8,
		sound: IdOr<SoundEvent>,
		has_particles: bool,
		effects: Vec<ConsumeEffect>,
	},
	UseRemainder(Slot),
	/// Group of items to apply the cooldown to. Only present if Has cooldown group is true; otherwise defaults to the item's identifier.
	UseCooldown {
		seconds: f32,
		group: Option<String>,
	},
	/// Parameter - Types, Tag specifying damage types the item is immune to. Not prefixed by '#'!.
	DamageResistant(String),
	Tool {
		rules: Vec<ToolRule>,
		default_mining_speed: f32,
		damage_per_block: i32,
	},
	Weapon {
		damage_per_attack: i32,
		disable_blocking_for_seconds: f32,
	},
	// Opaque internal value controlling how expensive enchantments may be offered.
	Enchantable(i32),
	Equippable {
		slot: u8,
		equip_sound: IdOr<SoundEvent>,
		model: Option<String>,
		camera_overlay: Option<String>,
		allowed_entities: Option<IdSet>,
		dispensable: bool,
		swappable: bool,
		damage_on_hurt: bool,
	},
	/// Items that can be combined with this item in an anvil to repair it.
	Repairable(IdSet),
	/// Makes the item function like elytra.
	Glider,
	TooltipStyle(String),
	/// Makes the item function like a totem of undying.
	DeathProtection(Vec<ConsumeEffect>),
	BlockAttacks {
		block_delay_seconds: f32,
		disable_cooldown_scale: f32,
		damage_reductions: Vec<DamageReduction>,
		item_damage_threshold: f32,
		item_damage_base: f32,
		item_damage_factor: f32,
		bypassed_by: Option<String>,
		block_sound: Option<IdOr<SoundEvent>>,
		disable_sound: Option<IdOr<SoundEvent>>,
	},
	/// The enchantments stored in this enchanted book.
	/// Key: The ID of the enchantment in the enchantment registry. \
	/// Value: The level of the enchantment.
	StoredEnchantments(Vec<(u64, i32)>),
	DyedColor(i32),
	MapColor(i32),
	MapId(i32),
	MapDecorations(DynNBT),
	/// Type of post processing. Can be either:
	/// - 0 - Lock
	/// - 1 - Scale
	MapPostProcessing(u8),
	/// Projectiles loaded into a charged crossbow.
	ChargedProjectiles(Vec<Slot>),
	BundleContents(Vec<Slot>),
	PotionContents {
		potion_id: Option<u64>,
		custom_color: Option<i32>,
		custom_effects: Vec<PotionEffect>,
		custom_name: String,
	},
	/// Parameter - Effect Multiplier
	PotionDurationScale(f32),
	/// Key - The ID of the effect in the potion effect type registry.
	/// Value - The duration of the effect.
	SuspiciousStewEffects(Vec<(u64, i32)>),
	/// Parameter - Pages
	/// Page:
	/// - The raw text of the page
	/// - The content after passing through chat filters
	WritableBookContent(Vec<(String, Option<String>)>),
	WrittenBookContent {
		raw_title: String,
		filtered_title: Option<String>,
		author: String,
		generation: i32,
		/// Page:
		/// - The raw text of the page
		/// - The content after passing through chat filters
		pages: Vec<(String, Option<String>)>,
		resolved: bool,
	},
	/// Armor's trim pattern and color
	Trim(IdOr<TrimMaterial>, IdOr<TrimPattern>),
	DebugStrickState(DynNBT),
	EntityData(DynNBT),
	BucketEntityData(DynNBT),
	BlockEntityData(DynNBT),
	Instrument(IdOr<Instrument>),
	ProvidesTrimMaterial(ProvidesTrimMaterial),
	/// Between 0 and 4.
	OminousBottleAmplifier(u8),
	JukeboxPlayable(JukeboxPlayable),
	/// A pattern identifier like #minecraft:pattern_item/globe
	ProvidesBannerPatterns(String),
	Recipes(DynNBT),
	LodestoneTracker {
		has_global_position: bool,
		dimension: String,
		position: (f64, f64, f64),
		tracked: bool,
	},
	FireworkExplosion(FireworkExplosion),
	Fireworks {
		flight_duration: i32,
		explosions: Vec<FireworkExplosion>,
	},
	Profile {
		name: Option<String>,
		unique_id: Option<Uuid>,
		properties: Vec<Property>,
	},
	NoteBlockSound(String),
	BannerPatterns(Vec<BannerLayer>),
	/// Can be one of the following:
	/// - 0 - White
	/// - 1 - Orange
	/// - 2 - Magenta
	/// - 3 - Light Blue
	/// - 4 - Yellow
	/// - 5 - Lime
	/// - 6 - Pink
	/// - 7 - Gray
	/// - 8 - Light Gray
	/// - 9 - Cyan
	/// - 10 - Purple
	/// - 11 - Blue
	/// - 12 - Brown
	/// - 13 - Green
	/// - 14 - Red
	/// - 15 - Black
	BaseColor(u8),
	/// The ID of the items in the item registry.
	PotDecorations([u64; 4]),
	/// Items inside a container of any type.
	Container(Vec<Slot>),
	BlockState(Vec<(String, String)>),
	Bees(Vec<HiveBee>),
	Lock(String),
	ContainerLoot(DynNBT),
	BreakSound(IdOr<SoundEvent>),
	VillagerVariant(u64),
	WolfVariant(u64),
	WolfSoundVariant(u64),
	WolfCollar(u8),
	/// 0: red, 1: snow
	FoxVariant(u8),
	/// 0: small, 1: medium, 2: large.
	SalmonSize(u8),
	ParrotVariant(u64),
	/// 0: kob, 1: sunstreak, 2: snooper, 3: dasher, 4: brinely, 5: spotty, 6: flopper, 7: stripey, 8: glitter, 9: blockfish, 10: betty, 11: clayfish.
	TropicalFishPattern(u8),
	/// Can be one of the following:
	/// - 0 - White
	/// - 1 - Orange
	/// - 2 - Magenta
	/// - 3 - Light Blue
	/// - 4 - Yellow
	/// - 5 - Lime
	/// - 6 - Pink
	/// - 7 - Gray
	/// - 8 - Light Gray
	/// - 9 - Cyan
	/// - 10 - Purple
	/// - 11 - Blue
	/// - 12 - Brown
	/// - 13 - Green
	/// - 14 - Red
	/// - 15 - Black
	TropicalFishBaseColor(u8),
	/// Can be one of the following:
	/// - 0 - White
	/// - 1 - Orange
	/// - 2 - Magenta
	/// - 3 - Light Blue
	/// - 4 - Yellow
	/// - 5 - Lime
	/// - 6 - Pink
	/// - 7 - Gray
	/// - 8 - Light Gray
	/// - 9 - Cyan
	/// - 10 - Purple
	/// - 11 - Blue
	/// - 12 - Brown
	/// - 13 - Green
	/// - 14 - Red
	/// - 15 - Black
	TropicalFishPatternColor(u8),
	/// 0: red, 1: brown.
	MooshroomVariant(u8),
	/// 0: brown, 1: white, 2: black, 3: white splotched, 4: gold, 5: salt, 6: evil.
	RabbitVariant(u8),
	PigVariant(u64),
	CowVariant(u64),
	ChickenVariant(ChickenVariant),
	FrogVariant(u64),
	/// 0: white, 1: creamy, 2: chestnut, 3: brown, 4: black, 5: gray, 6: dark brown.
	HorseVariant(u8),
	PaintingVariant(PaintingVariant),
	/// 0: creamy, 1: white, 2: brown, 3: gray.
	LlamaVariant(u8),
	/// 0: lucy, 1: wild, 2: gold, 3: cyan, 4: blue.
	AxolotlVariant(u8),
	CatVariant(u64),
	/// Can be one of the following:
	/// - 0 - White
	/// - 1 - Orange
	/// - 2 - Magenta
	/// - 3 - Light Blue
	/// - 4 - Yellow
	/// - 5 - Lime
	/// - 6 - Pink
	/// - 7 - Gray
	/// - 8 - Light Gray
	/// - 9 - Cyan
	/// - 10 - Purple
	/// - 11 - Blue
	/// - 12 - Brown
	/// - 13 - Green
	/// - 14 - Red
	/// - 15 - Black
	CatCollar(u8),
	/// Can be one of the following:
	/// - 0 - White
	/// - 1 - Orange
	/// - 2 - Magenta
	/// - 3 - Light Blue
	/// - 4 - Yellow
	/// - 5 - Lime
	/// - 6 - Pink
	/// - 7 - Gray
	/// - 8 - Light Gray
	/// - 9 - Cyan
	/// - 10 - Purple
	/// - 11 - Blue
	/// - 12 - Brown
	/// - 13 - Green
	/// - 14 - Red
	/// - 15 - Black
	SheepColor(u8),
	/// Can be one of the following:
	/// - 0 - White
	/// - 1 - Orange
	/// - 2 - Magenta
	/// - 3 - Light Blue
	/// - 4 - Yellow
	/// - 5 - Lime
	/// - 6 - Pink
	/// - 7 - Gray
	/// - 8 - Light Gray
	/// - 9 - Cyan
	/// - 10 - Purple
	/// - 11 - Blue
	/// - 12 - Brown
	/// - 13 - Green
	/// - 14 - Red
	/// - 15 - Black
	ShulkerColor(u8),
}

pub trait ReadWriteSlotComponent: DataReader + DataWriter {
	fn read_slot_component(&mut self) -> Result<StructuredComponent, ServerError>;
	fn write_slot_component(&mut self, val: &StructuredComponent) -> Result<(), ServerError>;
}

impl ReadWriteSlotComponent for Packet {
	fn read_slot_component(&mut self) -> Result<StructuredComponent, ServerError> {
		let _ = self.read_u16_varint()?; // id

		todo!()
	}
	fn write_slot_component(&mut self, val: &StructuredComponent) -> Result<(), ServerError> {
		self.write_usize_varint(val.enum_index())?;

		todo!()
	}
}

#[derive(Clone)]
pub struct Slot {
	pub id: i32,
	pub amount: i32,
	pub components: Vec<StructuredComponent>,
}

pub trait ReadWriteSlot: DataReader + DataWriter {
	fn read_slot(&mut self) -> Result<Option<Slot>, ServerError>;
	fn write_slot(&mut self, val: Option<Slot>) -> Result<(), ServerError>;
}

impl ReadWriteSlot for Packet {
	fn read_slot(&mut self) -> Result<Option<Slot>, ServerError> {
		let amount = self.read_varint()?;

		if amount > 0 {
			let id = self.read_varint()?;
			let components_len = self.read_varint()?;
			self.read_varint()?; // components_to_remove_len
			let mut components = Vec::new();
			for _ in 0..components_len {
				components.push(self.read_slot_component()?);
			}

			Ok(Some(Slot {
				id,
				amount,
				components,
			}))
		} else {
			Ok(None)
		}
	}
	fn write_slot(&mut self, val: Option<Slot>) -> Result<(), ServerError> {
		if let Some(val) = val {
			self.write_varint(val.amount)?;
			self.write_varint(val.id)?;
			self.write_usize_varint(val.components.len())?;
			let mut components_to_remove: Vec<u16> = (0..SLOT_COMPONENT_LENGTH).collect();
			self.write_usize_varint(SLOT_COMPONENT_LENGTH as usize - val.components.len())?;
			for comp in val.components {
				let id = comp.enum_index() as u16;
				self.write_slot_component(&comp)?;
				if let Some(index) = components_to_remove.iter().position(|v| *v == id) {
					components_to_remove.swap_remove(index);
				}
			}
			for id in components_to_remove {
				self.write_u16_varint(id)?;
			}
		} else {
			self.write_varint(0)?;
		}
		Ok(())
	}
}

#[derive(Clone)]
pub struct HashedSlot {
	pub id: i32,
	pub amount: i32,
	/// id -> crc32 hash
	pub components: Vec<(u16, i32)>,
}

pub trait ReadWriteHashedSlot: DataReader + DataWriter {
	fn read_hashed_slot(&mut self) -> Result<Option<HashedSlot>, ServerError>;
	fn write_hashed_slot(&mut self, val: Option<HashedSlot>) -> Result<(), ServerError>;
}

impl ReadWriteHashedSlot for Packet {
	fn read_hashed_slot(&mut self) -> Result<Option<HashedSlot>, ServerError> {
		let amount = self.read_varint()?;

		if amount > 0 {
			let id = self.read_varint()?;
			let components_len = self.read_varint()?;
			self.read_varint()?; // components_to_remove_len
			let mut components = Vec::new();
			for _ in 0..components_len {
				components.push((self.read_u16_varint()?, self.read_int()?));
			}

			Ok(Some(HashedSlot {
				id,
				amount,
				components,
			}))
		} else {
			Ok(None)
		}
	}
	fn write_hashed_slot(&mut self, val: Option<HashedSlot>) -> Result<(), ServerError> {
		if let Some(val) = val {
			self.write_varint(val.amount)?;
			self.write_varint(val.id)?;
			self.write_usize_varint(val.components.len())?;
			let mut components_to_remove: Vec<u16> = (0..SLOT_COMPONENT_LENGTH).collect();
			self.write_usize_varint(SLOT_COMPONENT_LENGTH as usize - val.components.len())?;
			for (id, hash) in val.components {
				self.write_u16_varint(id)?;
				self.write_int(hash)?;
				if let Some(index) = components_to_remove.iter().position(|v| *v == id) {
					components_to_remove.swap_remove(index);
				}
			}
			for id in components_to_remove {
				self.write_u16_varint(id)?;
			}
		} else {
			self.write_varint(0)?;
		}
		Ok(())
	}
}
