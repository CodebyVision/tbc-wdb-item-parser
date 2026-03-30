use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
struct ItemSpell {
  spell_id: u32,
  trigger: u32,
  charges: i32,
  cooldown: i32,
  category: u32,
  category_cooldown: i32,
}

#[derive(Debug, Clone, Copy)]
struct ItemDamage {
  min: f32,
  max: f32,
  dmg_type: u32,
}

#[derive(Debug, Clone, Copy)]
struct ItemStat {
  stat_type: i32,
  stat_value: i32,
}

#[derive(Debug, Clone)]
struct ItemEntry {
  entry: u32,
  class: u32,
  subclass: u32,
  name: String,
  displayid: u32,
  quality: u32,
  flags: u32,
  buy_price: u32,
  sell_price: u32,
  inventory_type: u32,
  allowable_class: i32,
  allowable_race: i32,
  item_level: u32,
  required_level: i32,
  required_skill: u32,
  required_skill_rank: u32,
  required_spell: u32,
  required_honor_rank: u32,
  required_city_rank: u32,
  required_reputation_faction: u32,
  required_reputation_rank: u32,
  maxcount: u32,
  stackable: u32,
  container_slots: u32,
  stats: [ItemStat; 10],
  damages: [ItemDamage; 5],
  armor: i32,
  holy_res: i32,
  fire_res: i32,
  nature_res: i32,
  frost_res: i32,
  shadow_res: i32,
  arcane_res: i32,
  delay: u32,
  ammo_type: u32,
  ranged_mod_range: f32,
  spells: [ItemSpell; 5],
  bonding: u32,
  description: String,
  page_text: u32,
  language_id: u32,
  page_material: u32,
  startquest: u32,
  lockid: u32,
  material: i32,
  sheath: u32,
  random_property: i32,
  block: u32,
  itemset: u32,
  max_durability: u32,
  area: u32,
  map: u32,
  bag_family: u32,
  totem_category: u32,
  socket_colors: [u32; 3],
  socket_contents: [u32; 3],
  socket_bonus: u32,
  gem_properties: u32,
  required_disenchant_skill: u32,
  armor_damage_modifier_bits: u32,
  disenchant_id: u32,
  duration: u32,
  extra_flags: u32,
}

fn read_u32_le(data: &[u8], pos: &mut usize) -> Result<u32, String> {
  if *pos + 4 > data.len() {
    return Err("unexpected EOF while reading u32".into());
  }
  let v = u32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
  *pos += 4;
  Ok(v)
}

fn read_i32_le(data: &[u8], pos: &mut usize) -> Result<i32, String> {
  Ok(read_u32_le(data, pos)? as i32)
}

fn read_f32_le(data: &[u8], pos: &mut usize) -> Result<f32, String> {
  if *pos + 4 > data.len() {
    return Err("unexpected EOF while reading f32".into());
  }
  let v = f32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
  *pos += 4;
  Ok(v)
}

fn read_cstring_utf8_lossy(data: &[u8], pos: &mut usize) -> Result<String, String> {
  let start = *pos;
  while *pos < data.len() && data[*pos] != 0 {
    *pos += 1;
  }
  if *pos >= data.len() {
    return Err("unterminated C-string".into());
  }
  let bytes = &data[start..*pos];
  // Many cache strings are effectively ANSI/UTF-8; `lossy` keeps export robust.
  let s = String::from_utf8_lossy(bytes).into_owned();
  *pos += 1; // skip null
  Ok(s)
}

fn sql_escape_str(s: &str) -> String {
  // Escape for MySQL using the simplest safe approach for our output style.
  // - escape single quotes by doubling them
  // - escape backslashes too (in case the user's SQL mode is permissive)
  let mut out = String::with_capacity(s.len());
  for ch in s.chars() {
    match ch {
      '\'' => out.push_str("''"),
      '\\' => out.push_str("\\\\"),
      '\r' => out.push_str("\\r"),
      '\n' => out.push_str("\\n"),
      _ => out.push(ch),
    }
  }
  out
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
  s.chars().take(max_chars).collect()
}

fn u8_clamp_from_i32(v: i32) -> u8 {
  if v <= 0 {
    0
  } else if v >= u8::MAX as i32 {
    u8::MAX
  } else {
    v as u8
  }
}

fn u16_clamp_from_i32(v: i32) -> u16 {
  if v <= 0 {
    0
  } else if v >= u16::MAX as i32 {
    u16::MAX
  } else {
    v as u16
  }
}

fn i16_clamp_from_i32(v: i32) -> i16 {
  if v < i16::MIN as i32 {
    i16::MIN
  } else if v > i16::MAX as i32 {
    i16::MAX
  } else {
    v as i16
  }
}

fn float_to_sql(f: f32) -> String {
  if !f.is_finite() {
    return "0".to_string();
  }
  // Keep it compact but stable enough for MySQL parsing.
  // `to_string()` is fine, but it can use scientific notation; trim trailing zeros instead.
  let s = format!("{}", f);
  s
}

fn parse_item_entry(data: &[u8], pos: &mut usize, build: u32) -> Result<Option<ItemEntry>, String> {
  if *pos + 8 > data.len() {
    return Ok(None);
  }

  let entry = read_u32_le(data, pos)?;
  if entry == 0 {
    return Ok(None);
  }

  let _entry_size = read_u32_le(data, pos)?; // used for validation in other tools; not required for our sequential parsing

  let class = read_u32_le(data, pos)?;
  let subclass = read_u32_le(data, pos)?;

  let mut name_slots: Vec<String> = Vec::with_capacity(4);
  for _ in 0..4 {
    name_slots.push(read_cstring_utf8_lossy(data, pos)?);
  }

  let displayid = read_u32_le(data, pos)?;
  let quality = read_u32_le(data, pos)?;
  let flags = read_u32_le(data, pos)?;
  let buy_price_raw = read_u32_le(data, pos)?;
  let sell_price = read_u32_le(data, pos)?;
  let inventory_type = read_u32_le(data, pos)?;
  let allowable_class = read_i32_le(data, pos)?;
  let allowable_race = read_i32_le(data, pos)?;
  let item_level = read_u32_le(data, pos)?;
  let required_level = read_i32_le(data, pos)?;
  let required_skill_raw = read_u32_le(data, pos)?;
  let required_skill_rank_raw = read_u32_le(data, pos)?;

  let mut required_spell = 0u32;
  let mut required_honor_rank = 0u32;
  let mut required_city_rank = 0u32;
  let mut required_reputation_faction = 0u32;
  let mut required_reputation_rank = 0u32;

  const CLIENT_BUILD_0_10_0: u32 = 3892;
  const CLIENT_BUILD_1_7_0: u32 = 4671;

  if build >= CLIENT_BUILD_0_10_0 {
    required_spell = read_u32_le(data, pos)?;
    required_honor_rank = read_u32_le(data, pos)?;
    required_city_rank = read_u32_le(data, pos)?;
  }
  if build >= CLIENT_BUILD_1_7_0 {
    required_reputation_faction = read_u32_le(data, pos)?;
    required_reputation_rank = read_u32_le(data, pos)?;
  }

  let maxcount = read_u32_le(data, pos)?;
  let stackable = read_u32_le(data, pos)?;
  let container_slots = read_u32_le(data, pos)?;

  let mut stats = [ItemStat {
    stat_type: 0,
    stat_value: 0,
  }; 10];
  for i in 0..10 {
    let stat_type = read_i32_le(data, pos)?;
    let stat_value = read_i32_le(data, pos)?;
    stats[i] = ItemStat {
      stat_type,
      stat_value,
    };
  }

  let mut damages = [ItemDamage {
    min: 0.0,
    max: 0.0,
    dmg_type: 0,
  }; 5];

  for i in 0..5 {
    let min = read_f32_le(data, pos)?; // build 0_10_0+ has float protodamage
    let max = read_f32_le(data, pos)?;
    let dmg_type = read_u32_le(data, pos)?;
    damages[i] = ItemDamage {
      min,
      max,
      dmg_type,
    };
  }

  let armor = read_i32_le(data, pos)?;
  let holy_res = read_i32_le(data, pos)?;
  let fire_res = read_i32_le(data, pos)?;
  let nature_res = read_i32_le(data, pos)?;
  let frost_res = read_i32_le(data, pos)?;
  let shadow_res = read_i32_le(data, pos)?;

  let arcane_res = {
    const CLIENT_BUILD_0_9_0: u32 = 3807;
    if build >= CLIENT_BUILD_0_9_0 {
      read_i32_le(data, pos)?
    } else {
      0
    }
  };

  let delay = read_u32_le(data, pos)?;
  let ammo_type = read_u32_le(data, pos)?;

  let ranged_mod_range = {
    const CLIENT_BUILD_1_10_0: u32 = 5195;
    if build >= CLIENT_BUILD_1_10_0 {
      read_f32_le(data, pos)?
    } else {
      0.0
    }
  };

  let mut spells = [ItemSpell {
    spell_id: 0,
    trigger: 0,
    charges: 0,
    cooldown: 0,
    category: 0,
    category_cooldown: 0,
  }; 5];

  for i in 0..5 {
    let spell_id = read_u32_le(data, pos)?;
    let trigger = read_u32_le(data, pos)?;
    let charges = read_i32_le(data, pos)?;
    let cooldown = read_i32_le(data, pos)?;
    let category = read_u32_le(data, pos)?;
    let category_cooldown = read_i32_le(data, pos)?;
    spells[i] = ItemSpell {
      spell_id,
      trigger,
      charges,
      cooldown,
      category,
      category_cooldown,
    };
  }

  let bonding = read_u32_le(data, pos)?;
  let description = truncate_chars(&read_cstring_utf8_lossy(data, pos)?, 255);

  let page_text = read_u32_le(data, pos)?;
  let language_id = read_u32_le(data, pos)?;
  let page_material = read_u32_le(data, pos)?;
  let startquest = read_u32_le(data, pos)?;
  let lockid = read_u32_le(data, pos)?;
  let material = read_i32_le(data, pos)?;
  let sheath = read_u32_le(data, pos)?;

  let random_property = {
    const CLIENT_BUILD_0_5_5: u32 = 3494;
    if build >= CLIENT_BUILD_0_5_5 {
      read_i32_le(data, pos)?
    } else {
      0
    }
  };

  let block = {
    const CLIENT_BUILD_0_6_0: u32 = 3592;
    if build >= CLIENT_BUILD_0_6_0 {
      read_u32_le(data, pos)?
    } else {
      0
    }
  };

  let itemset = {
    const CLIENT_BUILD_0_10_0: u32 = 3892;
    if build >= CLIENT_BUILD_0_10_0 {
      read_u32_le(data, pos)?
    } else {
      0
    }
  };

  let max_durability = {
    const CLIENT_BUILD_0_12_0: u32 = 3988;
    if build >= CLIENT_BUILD_0_12_0 {
      read_u32_le(data, pos)?
    } else {
      0
    }
  };

  let area = {
    const CLIENT_BUILD_1_7_0: u32 = 4671;
    if build >= CLIENT_BUILD_1_7_0 {
      read_u32_le(data, pos)?
    } else {
      0
    }
  };

  let map = {
    const CLIENT_BUILD_1_11_0: u32 = 5428;
    if build >= CLIENT_BUILD_1_11_0 {
      read_u32_le(data, pos)?
    } else {
      0
    }
  };

  let bag_family = {
    const CLIENT_BUILD_1_9_0: u32 = 4937;
    if build >= CLIENT_BUILD_1_9_0 {
      read_u32_le(data, pos)?
    } else {
      0
    }
  };

  // Extra DWORDs after bag_family (sockets/gem/disenchant/etc).
  // This tail ordering was inferred by aligning with a known-good sequential parse.
  let totem_category = read_u32_le(data, pos)?;
  let socket_c1 = read_u32_le(data, pos)?;
  let socket_cnt1 = read_u32_le(data, pos)?;
  let socket_c2 = read_u32_le(data, pos)?;
  let socket_cnt2 = read_u32_le(data, pos)?;
  let socket_c3 = read_u32_le(data, pos)?;
  let socket_cnt3 = read_u32_le(data, pos)?;
  let socket_bonus = read_u32_le(data, pos)?;
  let gem_properties = read_u32_le(data, pos)?;
  let armor_damage_modifier_bits = read_u32_le(data, pos)?;
  let required_disenchant_skill = read_u32_le(data, pos)?;
  let duration = read_u32_le(data, pos)?;
  let extra_flags = read_u32_le(data, pos)?;
  let disenchant_id = read_u32_le(data, pos)?;

  let name = truncate_chars(&name_slots[0], 255);
  let buy_price = if buy_price_raw == u32::MAX { 0 } else { buy_price_raw };

  Ok(Some(ItemEntry {
    entry,
    class,
    subclass,
    name,
    displayid,
    quality,
    flags,
    buy_price,
    sell_price,
    inventory_type,
    allowable_class,
    allowable_race,
    item_level,
    required_level,
    required_skill: if required_skill_raw == u32::MAX { 0 } else { required_skill_raw },
    required_skill_rank: if required_skill_rank_raw == u32::MAX {
      0
    } else {
      required_skill_rank_raw
    },
    required_spell,
    required_honor_rank,
    required_city_rank,
    required_reputation_faction,
    required_reputation_rank,
    maxcount,
    stackable,
    container_slots,
    stats,
    damages,
    armor,
    holy_res,
    fire_res,
    nature_res,
    frost_res,
    shadow_res,
    arcane_res,
    delay,
    ammo_type,
    ranged_mod_range,
    spells,
    bonding,
    description,
    page_text,
    language_id,
    page_material,
    startquest,
    lockid,
    material,
    sheath,
    random_property,
    block,
    itemset,
    max_durability,
    area,
    map,
    bag_family,
    totem_category,
    socket_colors: [socket_c1, socket_c2, socket_c3],
    socket_contents: [socket_cnt1, socket_cnt2, socket_cnt3],
    socket_bonus,
    gem_properties,
    required_disenchant_skill,
    armor_damage_modifier_bits,
    disenchant_id,
    duration,
    extra_flags,
  }))
}

fn sql_row_values(it: &ItemEntry) -> Vec<String> {
  let req_spell = if it.required_spell == u32::MAX { 0 } else { it.required_spell };
  let req_honor = if it.required_honor_rank == u32::MAX {
    0
  } else {
    it.required_honor_rank
  };
  let req_city = if it.required_city_rank == u32::MAX { 0 } else { it.required_city_rank };
  let req_rep_faction = if it.required_reputation_faction == u32::MAX {
    0
  } else {
    it.required_reputation_faction
  };
  let req_rep_rank = if it.required_reputation_rank == u32::MAX {
    0
  } else {
    it.required_reputation_rank
  };

  // cmangos schema uses SMALLINT UNSIGNED but expects `-1` as a sentinel in many places.
  let req_disenchant = if it.required_disenchant_skill == u32::MAX {
    -1
  } else {
    it.required_disenchant_skill.min(i16::MAX as u32) as i32
  };

  let armor_dmg_mod = f32::from_bits(it.armor_damage_modifier_bits);
  let armor_dmg_mod_sql = float_to_sql(armor_dmg_mod);

  let allowable_class_sql = it.allowable_class;
  let allowable_race_sql = it.allowable_race;

  let required_level_sql = if it.required_level < 0 {
    0
  } else {
    (it.required_level as u32).min(u8::MAX as u32)
  };

  let material_sql = if it.material < 0 { 0 } else { (it.material as u32).min(u8::MAX as u32) };

  let random_property_sql = if it.random_property < 0 {
    0
  } else {
    it.random_property as u32
  };

  let class_sql = if it.class == u32::MAX { 0 } else { it.class.min(u8::MAX as u32) };
  let subclass_sql = if it.subclass == u32::MAX { 0 } else { it.subclass.min(u8::MAX as u32) };
  let quality_sql = if it.quality == u32::MAX { 0 } else { it.quality.min(u8::MAX as u32) };
  let inventory_type_sql = if it.inventory_type == u32::MAX { 0 } else { it.inventory_type.min(u8::MAX as u32) };
  let item_level_sql = if it.item_level == u32::MAX { 0 } else { it.item_level.min(u8::MAX as u32) };
  let required_skill_sql = it.required_skill.min(u16::MAX as u32);
  let required_skill_rank_sql = it.required_skill_rank.min(u16::MAX as u32);
  let maxcount_sql = it.maxcount.min(u16::MAX as u32);
  let stackable_sql = if it.stackable == u32::MAX { 0 } else { it.stackable.min(u16::MAX as u32) };
  let container_slots_sql = if it.container_slots == u32::MAX { 0 } else { it.container_slots.min(u8::MAX as u32) };

  let stat_pairs: Vec<String> = (0..10)
    .flat_map(|i| {
      let st = it.stats[i].stat_type;
      let sv = it.stats[i].stat_value;
      let st_sql = u8_clamp_from_i32(st) as u32;
      let sv_sql = i16_clamp_from_i32(sv) as i32;
      vec![st_sql.to_string(), sv_sql.to_string()]
    })
    .collect();

  let dmg_triplets: Vec<String> = (0..5)
    .flat_map(|i| {
      let d = &it.damages[i];
      let dt_sql = u8_clamp_from_i32(d.dmg_type as i32) as u32;
      vec![float_to_sql(d.min), float_to_sql(d.max), dt_sql.to_string()]
    })
    .collect();

  let spell_triplets_and_more: Vec<String> = (0..5)
    .flat_map(|i| {
      let s = &it.spells[i];
      let spell_id_sql = s.spell_id;
      let trigger_sql = u8_clamp_from_i32(s.trigger as i32) as u32;
      let charges_sql = if s.charges < 0 { 0 } else { s.charges as u32 };
      let cooldown_sql = s.cooldown;
      let category_sql = u16_clamp_from_i32(s.category as i32) as u32;
      let category_cooldown_sql = s.category_cooldown;
      vec![
        spell_id_sql.to_string(),
        trigger_sql.to_string(),
        charges_sql.to_string(),
        cooldown_sql.to_string(),
        category_sql.to_string(),
        category_cooldown_sql.to_string(),
      ]
    })
    .collect();

  let armor_dmg_mod_sql = armor_dmg_mod_sql;

  let socket_colors = &it.socket_colors;
  let socket_contents = &it.socket_contents;

  vec![
    it.entry.to_string(),
    class_sql.to_string(),
    subclass_sql.to_string(),
    format!("'{}'", sql_escape_str(&it.name)),
    it.displayid.to_string(),
    quality_sql.to_string(),
    it.flags.to_string(),
    it.buy_price.to_string(),
    it.sell_price.to_string(),
    inventory_type_sql.to_string(),
    allowable_class_sql.to_string(),
    allowable_race_sql.to_string(),
    item_level_sql.to_string(),
    required_level_sql.to_string(),
    required_skill_sql.to_string(),
    required_skill_rank_sql.to_string(),
    req_spell.to_string(),
    req_honor.to_string(),
    req_city.to_string(),
    req_rep_faction.to_string(),
    req_rep_rank.to_string(),
    maxcount_sql.to_string(),
    stackable_sql.to_string(),
    container_slots_sql.to_string(),
    // stats (type/value x10)
    // damages (min/max/type x5)
    // spells (6-tuple x5)
  ]
  .into_iter()
  .chain(stat_pairs.into_iter())
  .chain(dmg_triplets.into_iter())
  .chain(vec![
    u16_clamp_from_i32(it.armor).to_string(),
    u8_clamp_from_i32(it.holy_res).to_string(),
    u8_clamp_from_i32(it.fire_res).to_string(),
    u8_clamp_from_i32(it.nature_res).to_string(),
    u8_clamp_from_i32(it.frost_res).to_string(),
    u8_clamp_from_i32(it.shadow_res).to_string(),
    u8_clamp_from_i32(it.arcane_res).to_string(),
    it.delay.to_string(),
    it.ammo_type.to_string(),
    float_to_sql(it.ranged_mod_range),
  ].into_iter())
  .chain(spell_triplets_and_more.into_iter())
  .chain(vec![
    it.bonding.to_string(),
    format!("'{}'", sql_escape_str(&it.description)),
    it.page_text.to_string(),
    (it.language_id.min(u8::MAX as u32)).to_string(),
    (it.page_material.min(u8::MAX as u32)).to_string(),
    it.startquest.to_string(),
    it.lockid.to_string(),
    material_sql.to_string(),
    it.sheath.to_string(),
    random_property_sql.to_string(),
    it.block.to_string(),
    it.itemset.to_string(),
    it.max_durability.to_string(),
    it.area.to_string(),
    it.map.to_string(),
    it.bag_family.to_string(),
    it.totem_category.min(u8::MAX as u32).to_string(),
    socket_colors[0].min(u8::MAX as u32).to_string(),
    socket_contents[0].min(16_777_215).to_string(),
    socket_colors[1].min(u8::MAX as u32).to_string(),
    socket_contents[1].min(16_777_215).to_string(),
    socket_colors[2].min(u8::MAX as u32).to_string(),
    socket_contents[2].min(16_777_215).to_string(),
    it.socket_bonus.min(16_777_215).to_string(),
    it.gem_properties.min(16_777_215).to_string(),
    req_disenchant.to_string(),
    armor_dmg_mod_sql,
    it.disenchant_id.to_string(),
    it.duration.to_string(),
    it.extra_flags.min(u8::MAX as u32).to_string(),
  ])
  .collect()
}

fn item_template_columns() -> Vec<String> {
  let mut cols: Vec<String> = Vec::new();

  // Basic identity / requirements
  cols.push("`entry`".to_string());
  cols.push("`class`".to_string());
  cols.push("`subclass`".to_string());
  cols.push("`name`".to_string());
  cols.push("`displayid`".to_string());
  cols.push("`Quality`".to_string());
  cols.push("`Flags`".to_string());
  cols.push("`BuyPrice`".to_string());
  cols.push("`SellPrice`".to_string());
  cols.push("`InventoryType`".to_string());
  cols.push("`AllowableClass`".to_string());
  cols.push("`AllowableRace`".to_string());
  cols.push("`ItemLevel`".to_string());
  cols.push("`RequiredLevel`".to_string());
  cols.push("`RequiredSkill`".to_string());
  cols.push("`RequiredSkillRank`".to_string());
  cols.push("`requiredspell`".to_string());
  cols.push("`requiredhonorrank`".to_string());
  cols.push("`RequiredCityRank`".to_string());
  cols.push("`RequiredReputationFaction`".to_string());
  cols.push("`RequiredReputationRank`".to_string());
  cols.push("`maxcount`".to_string());
  cols.push("`stackable`".to_string());
  cols.push("`ContainerSlots`".to_string());

  // Stats (stat_type1..10, stat_value1..10)
  for i in 1..=10 {
    cols.push(format!("`stat_type{i}`"));
    cols.push(format!("`stat_value{i}`"));
  }

  // Damage (dmg_min1..5, dmg_max1..5, dmg_type1..5)
  for i in 1..=5 {
    cols.push(format!("`dmg_min{i}`"));
    cols.push(format!("`dmg_max{i}`"));
    cols.push(format!("`dmg_type{i}`"));
  }

  // Resistances / weapon params
  cols.push("`armor`".to_string());
  cols.push("`holy_res`".to_string());
  cols.push("`fire_res`".to_string());
  cols.push("`nature_res`".to_string());
  cols.push("`frost_res`".to_string());
  cols.push("`shadow_res`".to_string());
  cols.push("`arcane_res`".to_string());
  cols.push("`delay`".to_string());
  cols.push("`ammo_type`".to_string());
  cols.push("`RangedModRange`".to_string());

  // Spells
  for i in 1..=5 {
    cols.push(format!("`spellid_{i}`"));
    cols.push(format!("`spelltrigger_{i}`"));
    cols.push(format!("`spellcharges_{i}`"));
    cols.push(format!("`spellcooldown_{i}`"));
    cols.push(format!("`spellcategory_{i}`"));
    cols.push(format!("`spellcategorycooldown_{i}`"));
  }

  // Rest
  cols.push("`bonding`".to_string());
  cols.push("`description`".to_string());
  cols.push("`PageText`".to_string());
  cols.push("`LanguageID`".to_string());
  cols.push("`PageMaterial`".to_string());
  cols.push("`startquest`".to_string());
  cols.push("`lockid`".to_string());
  cols.push("`Material`".to_string());
  cols.push("`sheath`".to_string());
  cols.push("`RandomProperty`".to_string());
  cols.push("`block`".to_string());
  cols.push("`itemset`".to_string());
  cols.push("`MaxDurability`".to_string());
  cols.push("`area`".to_string());
  cols.push("`Map`".to_string());
  cols.push("`BagFamily`".to_string());

  // Sockets / gems / disenchant / duration
  cols.push("`TotemCategory`".to_string());
  cols.push("`socketColor_1`".to_string());
  cols.push("`socketContent_1`".to_string());
  cols.push("`socketColor_2`".to_string());
  cols.push("`socketContent_2`".to_string());
  cols.push("`socketColor_3`".to_string());
  cols.push("`socketContent_3`".to_string());
  cols.push("`socketBonus`".to_string());
  cols.push("`GemProperties`".to_string());
  cols.push("`RequiredDisenchantSkill`".to_string());
  cols.push("`ArmorDamageModifier`".to_string());
  cols.push("`DisenchantID`".to_string());
  cols.push("`Duration`".to_string());
  cols.push("`ExtraFlags`".to_string());

  cols
}

pub fn export_itemcache_to_cmangos_item_template_sql(
  itemcache_path: &Path,
  output_sql_path: &Path,
) -> Result<usize, String> {
  let data = std::fs::read(itemcache_path).map_err(|e| e.to_string())?;
  if data.len() < 24 {
    return Err("itemcache.wdb is too small".into());
  }

  let sig = &data[0..4];
  if sig != b"BDIW" {
    return Err(format!("unexpected signature: {:?}", sig));
  }

  let build = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
  // The file name/size you provided (2.4.3 build 8606) is what this exporter targets.
  // We still keep build-based conditional reads where vmangos does.

  let mut pos: usize = 24;

  let out_file = File::create(output_sql_path).map_err(|e| e.to_string())?;
  let mut writer = BufWriter::new(out_file);

  let columns = item_template_columns();
  let columns_joined = columns.join(", ");

  writer
    .write_all(format!("REPLACE INTO `item_template` ({columns_joined}) VALUES\n").as_bytes())
    .map_err(|e| e.to_string())?;

  let mut first_row = true;
  let mut written = 0usize;

  loop {
    match parse_item_entry(&data, &mut pos, build) {
      Ok(Some(it)) => {
        // Some cache entries are placeholders; exporting them is rarely helpful.
        if it.inventory_type == 0 {
          continue;
        }

        let values = sql_row_values(&it);
        if values.len() != columns.len() {
          return Err(format!(
            "internal error: value/column length mismatch: {} values vs {} columns",
            values.len(),
            columns.len()
          ));
        }

        if !first_row {
          writer.write_all(b",\n").map_err(|e| e.to_string())?;
        }
        first_row = false;
        writer
          .write_all(format!("({})", values.join(", ")).as_bytes())
          .map_err(|e| e.to_string())?;
        written += 1;
      }
      Ok(None) => break,
      Err(e) => {
        // Make parsing failures obvious to the user.
        return Err(e);
      }
    }
  }

  writer.write_all(b";\n").map_err(|e| e.to_string())?;
  writer.flush().map_err(|e| e.to_string())?;

  Ok(written)
}

