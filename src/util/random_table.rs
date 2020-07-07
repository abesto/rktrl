use bracket_lib::prelude::RandomNumberGenerator;

pub struct RandomEntry<T> {
    item: T,
    weight: i32,
}

impl<T> RandomEntry<T> {
    pub fn new(item: T, weight: i32) -> RandomEntry<T> {
        RandomEntry { item, weight }
    }
}

#[derive(Default)]
pub struct RandomTable<T> {
    entries: Vec<RandomEntry<T>>,
    total_weight: i32,
}

impl<T> RandomTable<T>
where
    T: Copy,
{
    pub fn new() -> RandomTable<T> {
        RandomTable {
            entries: Vec::new(),
            total_weight: 0,
        }
    }

    pub fn add(mut self, item: T, weight: i32) -> RandomTable<T> {
        self.total_weight += weight;
        self.entries.push(RandomEntry::new(item, weight));
        self
    }

    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> Option<T> {
        if self.total_weight == 0 {
            return None;
        }
        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        let mut index: usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return Some(self.entries[index].item);
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        None
    }
}
