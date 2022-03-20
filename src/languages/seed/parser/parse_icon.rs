use crate::util::Icon;

impl Icon {
    pub fn parse(icon: &str) -> Result<Icon, String> {
        let mut icon_parts = icon.splitn(2, ':');

        let icon_type = icon_parts.next().unwrap();
        let icon_id = icon_parts.next().ok_or_else(|| String::from("invalid wheel icon syntax"))?;

        let icon = if icon_type == "file" {
            Icon::File(icon_id.to_owned())
        } else {
            let icon_id: u16 = icon_id.parse().map_err(|_| String::from("invalid wheel icon id"))?;
            match icon_type {
                "shard" => Icon::Shard(icon_id),
                "spell" => Icon::Spell(icon_id),
                "opher" => Icon::Opher(icon_id),
                "lupo" => Icon::Lupo(icon_id),
                "grom" => Icon::Grom(icon_id),
                "tuley" => Icon::Tuley(icon_id),
                _ => return Err(String::from("invalid wheel icon type")),
            }
        };

        if icon_parts.next().is_some() { return Err(String::from("too many parts")); }

        Ok(icon)
    }
}
