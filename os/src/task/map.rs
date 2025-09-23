#[derive(Debug, Clone, Copy)]
struct Entry {
    key: usize,
    value: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Map {
    entrys: [Option<Entry>; 10],
}

impl Map {
    pub fn new() -> Map {
        Self { entrys: [None; 10] }
    }

    pub fn inc(&mut self, key: usize) {
        for en in &mut self.entrys {
            if let Some(e) = en {
                if e.key == key {
                    e.value += 1;
                    return;
                }
            }
        }

        for pair in &mut self.entrys {
            if pair.is_none() {
                *pair = Some(Entry { key, value: 1 });
                return;
            }
        }
    }

    pub fn get(&self, key: usize) -> usize {
        for en in &self.entrys {
            if let Some(e) = en {
                if e.key == key {
                    return e.value;
                }
            }
        }
        0
    }
}
