
struct record {
	u8 tag;
	u64 key_len;
	u8 key[key_len];
	
	if (tag == 'S') {
		u8 value_tag;
		if (value_tag == 'B') {
			u64 value_len;
			u8 value[value_len];
		}
	}
	else if (tag == 'A') {
		u64 value_len;
		u8 value[value_len];
	}
};

record r1[6] @ 0x00;

