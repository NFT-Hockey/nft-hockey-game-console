use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use crate::team::players::player::{Hand, PlayerRole, PlayerType};
use crate::PlayerPosition::*;
use crate::user_info::UserId;


#[derive(Clone, BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FieldPlayer {
    pub id: Option<TokenId>,
    pub img: Option<SRC>,
    pub name: Option<String>,
    pub teamwork: Option<f32>,
    pub number_of_penalty_events: Option<u8>,

    pub reality: bool,
    pub nationality: String,
    pub birthday: u64,
    pub player_type: PlayerType,

    pub number: u8,
    pub hand: Hand,
    pub player_role: PlayerRole,
    pub native_position: PlayerPosition,
    pub stats: FieldPlayerStats,

    pub user_id: Option<UserId>,
}

impl FieldPlayer {
    pub fn get_user_id(&self) -> usize { self.user_id.clone().unwrap() }

    pub fn get_player_id(&self) -> TokenId {
        self.id.clone().unwrap()
    }

    pub fn get_position_coefficient(&self, position: &PlayerPosition) -> f32 {
        let native_pos = 1.0 as f32;
        let other_edge = 0.95 as f32;
        let another_pos = 0.8 as f32;
        let center = 0.75 as f32;

        match position {
            Center => match self.native_position {
                Center => native_pos,
                RightWing => another_pos,
                LeftWing => another_pos,
                LeftDefender => another_pos,
                RightDefender => another_pos,
                _ => panic!("Native position not set")
            },
            RightWing => match self.native_position {
                Center =>  center,
                RightWing => native_pos,
                LeftWing => other_edge,
                LeftDefender => another_pos,
                RightDefender => another_pos,
                _ => panic!("Native position not set")
            },
            LeftWing => match self.native_position {
                Center => center,
                RightWing => other_edge,
                LeftWing => native_pos,
                LeftDefender => another_pos,
                RightDefender => another_pos,
                _ => panic!("Native position not set")
            },
            RightDefender => match self.native_position {
                Center => center,
                RightWing =>  another_pos,
                LeftWing => another_pos,
                LeftDefender => other_edge,
                RightDefender => native_pos,
                _ => panic!("Native position not set")
            },
            LeftDefender => match self.native_position {
                Center => center,
                RightWing => another_pos,
                LeftWing => another_pos,
                LeftDefender => native_pos,
                RightDefender => other_edge,
                _ => panic!("Native position not set")
            },
            _ => panic!("Position not set")
        }
    }
}

#[derive(Clone, Copy, BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FieldPlayerStats {
    // Skating
    pub acceleration: u8,
    pub agility: u8,
    pub balance: u8,
    pub endurance: u8,
    pub speed: u8,

    // Shooting
    pub slap_shot_accuracy: u8,
    pub slap_shot_power: u8,
    pub wrist_shot_accuracy: u8,
    pub wrist_shot_power: u8,

    // StickHandling
    pub deking: u8,
    pub hand_eye: u8,
    pub passing: u8,
    pub puck_control: u8,

    // Strength
    pub aggressiveness: u8,
    pub body_checking: u8,
    pub durability: u8,
    pub fighting_skill: u8,
    pub strength: u8,

    // IQ
    pub discipline: u8,
    pub offensive: u8,
    pub poise: u8,
    pub morale: u8,

    // Defense
    pub defensive_awareness: u8,
    pub face_offs: u8,
    pub shot_blocking: u8,
    pub stick_checking: u8,
}

impl FieldPlayerStats {
    pub fn get_skating(&self) -> f32 {
        (self.acceleration as f32 +
            self.agility as f32 +
            self.balance as f32 +
            self.endurance as f32 +
            self.speed as f32) / 5 as f32
    }

    pub fn get_shooting(&self) -> f32 {
        (self.slap_shot_accuracy as f32 +
            self.slap_shot_power as f32 +
            self.wrist_shot_accuracy as f32 +
            self.wrist_shot_power as f32) / 4 as f32
    }

    pub fn get_stick_handling(&self) -> f32 {
        (self.deking as f32 +
            self.hand_eye as f32 +
            self.passing as f32 +
            self.puck_control as f32) / 4 as f32
    }

    pub fn get_strength(&self) -> f32 {
        (self.aggressiveness as f32 +
            self.body_checking as f32 +
            self.durability as f32 +
            self.fighting_skill as f32 +
            self.strength as f32) / 5 as f32
    }

    pub fn get_iq(&self) -> f32 {
        (self.discipline as f32 +
            self.offensive as f32 +
            self.poise as f32 +
            self.morale as f32) / 4 as f32
    }

    pub fn get_defense(&self) -> f32 {
        (self.defensive_awareness as f32 +
            self.face_offs as f32 +
            self.shot_blocking as f32 +
            self.stick_checking as f32) / 4 as f32
    }

    pub fn increase_strength(&mut self, value: u8) {
        self.aggressiveness += value;
        self.body_checking += value;
        self.durability += value;
        self.fighting_skill += value;
        self.strength += value;
    }

    pub fn increase_iq(&mut self, value: u8) {
        self.discipline += value;
        self.offensive += value;
        self.poise += value;
        self.morale += value;
    }

    pub fn decrease_strength(&mut self, value: u8) {
        self.aggressiveness -= value;
        self.body_checking -= value;
        self.durability -= value;
        self.fighting_skill -= value;
        self.strength -= value;
    }

    pub fn decrease_iq(&mut self, value: u8) {
        self.discipline -= value;
        self.offensive -= value;
        self.poise -= value;
        self.morale -= value;
    }

    pub fn get_discipline(&self) -> f32 {
        self.discipline as f32
    }
}
