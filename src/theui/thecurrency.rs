const BRONZE_TO_SILVER: u32 = 10;
const SILVER_TO_GOLD: u32 = 10;
const BRONZE_TO_GOLD: u32 = BRONZE_TO_SILVER * SILVER_TO_GOLD;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TheCurrency {
    gold: u16,
    silver: u16,
    bronze: u16,
}

impl TheCurrency {
    // Constructor to create new instances of TheCurrency.
    pub fn new(gold: u16, silver: u16, bronze: u16) -> Self {
        let total_bronze =
            bronze as u32 + (silver as u32 * BRONZE_TO_SILVER) + (gold as u32 * BRONZE_TO_GOLD);
        Self {
            gold: (total_bronze / BRONZE_TO_GOLD) as u16,
            silver: ((total_bronze % BRONZE_TO_GOLD) / BRONZE_TO_SILVER) as u16,
            bronze: (total_bronze % BRONZE_TO_SILVER) as u16,
        }
    }

    // Convert the currency to its bronze equivalent.
    pub fn to_bronze(&self) -> u32 {
        self.bronze as u32
            + (self.silver as u32 * BRONZE_TO_SILVER)
            + (self.gold as u32 * BRONZE_TO_GOLD)
    }

    // Check if the currency can afford a certain cost.
    pub fn can_afford(&self, other: &Self) -> bool {
        self.to_bronze() >= other.to_bronze()
    }
}

// Implementing addition for TheCurrency.
use std::ops::{Add, Sub};

impl Add for TheCurrency {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(0, 0, (self.to_bronze() + other.to_bronze()) as u16)
    }
}

// Implementing subtraction for TheCurrency.
impl Sub for TheCurrency {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            0,
            0,
            self.to_bronze()
                .checked_sub(other.to_bronze())
                .expect("Insufficient funds to subtract.") as u16,
        )
    }
}

// Implementing comparison traits for TheCurrency.
use std::cmp::Ordering;

impl PartialOrd for TheCurrency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.to_bronze().cmp(&other.to_bronze()))
    }
}

impl Ord for TheCurrency {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_bronze().cmp(&other.to_bronze())
    }
}
