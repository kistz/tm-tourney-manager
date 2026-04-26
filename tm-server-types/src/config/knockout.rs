use std::collections::BTreeMap;

use crate::config::{RoundsPerMap, helper::FinishTimeout};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct Knockout {
    pub finish_timeout: FinishTimeout,
    pub rounds_per_map: RoundsPerMap,
    pub rounds_without_elimination: i32,
    pub eliminated_player_number_rank: Vec<i32>,
}

impl Knockout {
    pub fn into_xml(&self) -> String {
        format!(
            r#"
        <setting name="S_RoundsPerMap" value="{}" type="integer"/>
        <setting name="S_FinishTimeout" value="{}" type="integer"/>
        <setting name="S_RoundsWithoutElimination" value="{}" type="integer"/>
        <setting name="S_EliminatedPlayersNbRanks" value="{}" type="text"/>
        <setting name="S_PointsRepartition" value="" type="text"/>
            "#,
            Into::<i32>::into(self.rounds_per_map),
            Into::<i32>::into(self.finish_timeout),
            self.rounds_without_elimination,
            eliminated_player_number_rank_format(&self.eliminated_player_number_rank)
        )
    }

    pub(super) fn get_xml_map(&self) -> BTreeMap<String, dxr::Value> {
        let mut map = BTreeMap::new();

        map.insert(
            "S_RoundsPerMap".into(),
            dxr::Value::Integer(Into::<i32>::into(self.rounds_per_map)),
        );
        map.insert(
            "S_EliminatedPlayersNbRanks".into(),
            dxr::Value::String(eliminated_player_number_rank_format(
                &self.eliminated_player_number_rank,
            )),
        );
        map.insert(
            "S_FinishTimeout".into(),
            dxr::Value::Integer(Into::<i32>::into(self.finish_timeout)),
        );
        map.insert(
            "S_RoundsWithoutElimination".into(),
            dxr::Value::Integer(Into::<i32>::into(self.rounds_without_elimination)),
        );
        map.insert("S_PointsRepartition".into(), dxr::Value::String("".into()));

        map
    }
}

fn eliminated_player_number_rank_format(points: &Vec<i32>) -> String {
    let mut string = String::new();
    for point in points {
        string += &point.to_string();
        string += ","
    }
    string.trim_end_matches(",").to_string()
}

impl Default for Knockout {
    fn default() -> Self {
        Self {
            finish_timeout: FinishTimeout::BasedOnMedal,
            eliminated_player_number_rank: vec![4, 16, 16],
            rounds_per_map: RoundsPerMap::Unlimited,
            rounds_without_elimination: 1,
        }
    }
}
