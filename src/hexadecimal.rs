/// Converts hexadecimal digit (0..F) to u8
pub fn hex_to_u8(hex: &str) -> u8 {
	return if let Ok(result) = hex.parse::<u8>() {
		result
	} else {
		match hex {
			"A" => 10,
		    "B" => 11,
		    "C" => 12,
		    "D" => 13,
		    "E" => 14,
		    "F" => 15,
			_ => panic!("Invalid hex digit: {}", hex),
		}
	};
}

/// Converts hexadecimal digit pair (00..FF) to u8
/// Used for hexadecimal to RGBA conversion
pub fn hex_pair_to_u8(hex_pair: &str) -> u8 {
	hex_to_u8(&hex_pair[0..1]) * 16 + hex_to_u8(&hex_pair[1..2])
}