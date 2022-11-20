# Format

- Header (16B)
- Displacements (n_disps \* 8B)
- Entries
- Entry data

# Header

- key (8B)
- n_disps (4B)
- n_entries (4B)

# Displacement

- a (4B)
- b (4B)

# Entry

- i (3B)
- k_len (1B)
- v_len (1B)

# Key encoding

The key (kanji) is utf16-encoded to save 1byte per char

# Value encoding

**Characters:**

0xxxxxxx: **H**iragana (unicode character: 0x3041 + x)
01111111: Hiragana prolonged sound mark (`ー`, unic: 0x30fc)
10000000: **S**eparator between reading and context (0x80)
11111111: **|** Separator between readings (0xff)
1xxxxxxx: Tail (ASCII character)

The default reading should be placed at the end
as it has the lowest priority. This way we can stop
iterating over the readings once a match is found.

**with tails:**

```
"描": {
  "k": "か",
  "": "びょう",
  "i": "か",
}
```

`kか|iか|びょう`

**with context:**

```
"色": {
  "": "いろ",
  "そん": "しょく",
}
```

`しょくSそん|いろ`
