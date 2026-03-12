/**
 * tamil-terms.js — Glossary data for Tamil prosody terms.
 * Used for contextual tooltips and educational popups.
 */

export const GLOSSARY = {
  ezhuthu: {
    tamil: 'எழுத்து',
    english: 'Grapheme / Letter',
    definition: 'The smallest written unit in Tamil. Four types exist: Uyir (vowels), Mei (consonants), Uyirmei (consonant+vowel combinations), and Aytham (the special character ஃ).',
    types: {
      uyir: {
        tamil: 'உயிர்',
        english: 'Vowel',
        definition: 'Pure vowels (அ, ஆ, இ, ஈ, etc.). Tamil has 12 vowels: 5 short (kuril) and 7 long (nedil).',
      },
      mei: {
        tamil: 'மெய்',
        english: 'Consonant',
        definition: 'Pure consonants with the pulli dot (க், ங், ச், etc.). Tamil has 18 consonants.',
      },
      uyirmei: {
        tamil: 'உயிர்மெய்',
        english: 'Consonant-Vowel',
        definition: 'A consonant combined with a vowel sign (க, கா, கி, etc.). These form the majority of Tamil graphemes.',
      },
      aytham: {
        tamil: 'ஆய்தம்',
        english: 'Aytham',
        definition: 'The special character ஃ. A unique Tamil letter that is neither vowel nor consonant.',
      },
    },
  },

  alavu: {
    tamil: 'அளவு',
    english: 'Vowel Length',
    definition: 'The duration of a vowel sound — either short (kuril) or long (nedil). This is fundamental to Tamil prosody.',
    types: {
      kuril: {
        tamil: 'குறில்',
        english: 'Short vowel',
        definition: 'Short vowels: அ, இ, உ, எ, ஒ. Each takes 1 matrai (time unit) when open.',
      },
      nedil: {
        tamil: 'நெடில்',
        english: 'Long vowel',
        definition: 'Long vowels: ஆ, ஈ, ஊ, ஏ, ஓ, ஐ, ஔ. Each takes 2 matrai (time units) when open.',
      },
    },
  },

  asai: {
    tamil: 'அசை',
    english: 'Mora / Metrical Unit',
    definition: 'The smallest rhythmic unit in Tamil prosody. A syllable or group of syllables that forms one beat. Two types exist: Neer and Nirai.',
    types: {
      neer: {
        tamil: 'நேர்',
        english: 'Neer (straight)',
        definition: 'A single syllable that is either a nedil (long vowel) or a kuril followed by a consonant (closed syllable). Represents a "heavy" beat.',
      },
      nirai: {
        tamil: 'நிரை',
        english: 'Nirai (combined)',
        definition: 'A kuril (short vowel, open syllable) followed by another syllable. The two syllables together form one metrical unit.',
      },
    },
  },

  seer: {
    tamil: 'சீர்',
    english: 'Metrical Foot',
    definition: 'A word-level prosodic unit consisting of 2-3 asai. Tamil prosody defines 12 named patterns (vaaipadu): 4 Iyarseer (2-asai) and 8 Venseer (3-asai).',
    types: {
      // Iyarseer (2-asai)
      thema: { tamil: 'தேமா', pattern: 'Neer + Neer', category: 'Iyarseer' },
      pulima: { tamil: 'புளிமா', pattern: 'Nirai + Neer', category: 'Iyarseer' },
      koovilam: { tamil: 'கூவிளம்', pattern: 'Neer + Nirai', category: 'Iyarseer' },
      karuvilam: { tamil: 'கருவிளம்', pattern: 'Nirai + Nirai', category: 'Iyarseer' },
      // Venseer (3-asai)
      themangai: { tamil: 'தேமாங்காய்', pattern: 'Neer + Neer + Neer', category: 'Venseer' },
      themangani: { tamil: 'தேமாங்கனி', pattern: 'Neer + Neer + Nirai', category: 'Venseer' },
      koovilankai: { tamil: 'கூவிளங்காய்', pattern: 'Neer + Nirai + Neer', category: 'Venseer' },
      koovilankani: { tamil: 'கூவிளங்கனி', pattern: 'Neer + Nirai + Nirai', category: 'Venseer' },
      pulimangai: { tamil: 'புளிமாங்காய்', pattern: 'Nirai + Neer + Neer', category: 'Venseer' },
      pulimangani: { tamil: 'புளிமாங்கனி', pattern: 'Nirai + Neer + Nirai', category: 'Venseer' },
      karuvilangai: { tamil: 'கருவிளங்காய்', pattern: 'Nirai + Nirai + Neer', category: 'Venseer' },
      karuvilankani: { tamil: 'கருவிளங்கனி', pattern: 'Nirai + Nirai + Nirai', category: 'Venseer' },
    },
  },

  thalai: {
    tamil: 'தளை',
    english: 'Junction / Binding',
    definition: 'The prosodic connection between consecutive words (seer). Valid junctions require alternating asai types at the boundary: the last asai of one word should differ from the first asai of the next.',
  },

  vendalai: {
    tamil: 'வெண்டளை',
    english: 'Venba Junction',
    definition: 'The specific junction rules for Venba meter. For Iyarseer words, the last asai must alternate with the first asai of the next word. For Venseer words, the kaaimun-ner pattern applies.',
  },

  ani: {
    tamil: 'அணி',
    english: 'Ornamentation',
    definition: 'Poetic ornaments that enhance the beauty of verse. Three main types in Kural Venba:',
    types: {
      etukai: {
        tamil: 'எதுகை',
        english: 'Second-letter Rhyme',
        definition: 'The second grapheme of the first word of each line shares the same consonant. A key structural ornament in Tamil poetry.',
      },
      monai: {
        tamil: 'மோனை',
        english: 'First-letter Alliteration',
        definition: 'The first grapheme of certain words in a line share the same consonant or vowel group. Typically between the 1st and 3rd words.',
      },
      iyaipu: {
        tamil: 'இயைபு',
        english: 'End Rhyme',
        definition: 'The final sound of the last word of each line matches. This creates an aural connection between line endings.',
      },
    },
  },

  matrai: {
    tamil: 'மாத்திரை',
    english: 'Time Unit / Duration',
    definition: 'The metrical weight of a syllable. Kuril-open = 1 matrai, Kuril-closed or Nedil-open = 2 matrai, Nedil-closed = 3 matrai.',
  },

  adi: {
    tamil: 'அடி',
    english: 'Line / Verse Line',
    definition: 'A line of poetry. A Kural has exactly 2 lines: the first line has 4 seer (words) and the second has 3.',
  },

  sol: {
    tamil: 'சொல்',
    english: 'Word',
    definition: 'A single word in the verse. Each word is classified by its seer type based on its asai pattern.',
  },

  kural: {
    tamil: 'குறள்',
    english: 'Kural Venba',
    definition: 'A specific type of Venba (Tamil verse form) with exactly 2 lines and 7 words (4+3). The Thirukkural is the most famous collection of 1330 such couplets.',
  },
};
