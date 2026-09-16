use std::sync::Mutex;
use ulid::Generator;

static GENERATOR: Mutex<Option<Generator>> = Mutex::new(None);

pub fn generate_id() -> String {
    let mut guard = GENERATOR.lock().unwrap_or_else(|p| p.into_inner());
    let gen = guard.get_or_insert_with(Generator::new);
    gen.generate()
        .expect("monotonic ULID generation")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_uniqueness_and_ordering() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
        assert!(id2 > id1, "Monotonic ULID guarantees id2 > id1");
    }
}
