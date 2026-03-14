In the Seer Layer, the rule engine transforms the raw asai_seq into metric identities. These fields are the building blocks for the next layer (Thalai).
Seer Layer Fields & Logic

| Field | Logic / Calculation | Possible Values |
|---|---|---|
| asai_count | Length of the asai_seq array. | 1, 2, 3, 4 |
| vaaippaadu | Mapping the asai_seq pattern to its classic name. | Thema, Pulima, Koovila, Karuvila, etc. |
| seer_group | Categorizing the foot based on the last syllable and count. | Ma-seer, Vila-seer, Kaai-seer, Kani-seer |
| eetru_pattern | (Only for the very last word) Mapping asai_seq to the four Venba endings. | Naal, Malar, Kaasu, Pirappu |
| is_kutriyalukaram | Check if kadai_ezhuthu is a shortened 'u' (கு, சு, டு, து, பு, று). | Boolean |

------------------------------
Detailed Mapping Logic1. Vaaippaadu & Seer Group Mapping
The engine uses a lookup table based on the asai_seq:

* Eerasai Seer (2 Syllables):
* ["Ner", "Ner"] $\rightarrow$ Thema (Group: Ma-seer)
   * ["Nirai", "Ner"] $\rightarrow$ Pulima (Group: Ma-seer)
   * ["Ner", "Nirai"] $\rightarrow$ Koovila (Group: Vila-seer)
   * ["Nirai", "Nirai"] $\rightarrow$ Karuvila (Group: Vila-seer)
* Moovasai Seer (3 Syllables):
* If ends in "Ner" $\rightarrow$ Kaai-seer (e.g., ["Ner", "Ner", "Ner"] is Themankai)
   * If ends in "Nirai" $\rightarrow$ Kani-seer (e.g., ["Nirai", "Nirai", "Nirai"] is Karuvilankani)

2. Eetru Seer (Final Word) Logic
This is specific to Venba classification. The engine ignores the standard group and applies this logic to the last word of the poem:

   1. If asai_count == 1:
   * ["Ner"] $\rightarrow$ Naal
      * ["Nirai"] $\rightarrow$ Malar
   2. If asai_count == 2:
   * ["Ner", "Ner"] AND is_kutriyalukaram == true $\rightarrow$ Kaasu
      * ["Nirai", "Ner"] AND is_kutriyalukaram == true $\rightarrow$ Pirappu
      * Note: If is_kutriyalukaram is false, it stays a standard Ma-seer.
   
3. Kutriyalukaram Logic
This is a pre-requisite for the eetru_pattern.

* Logic: If the kadai_ezhuthu belongs to the set {கு, சு, டு, து, பு, று} AND the word has more than one syllable (or a single long syllable), flag as true.

------------------------------
Why this Layer is Critical
If the Seer Layer identifies even one Kani-seer, the rule engine can immediately flag the poem as "Not a Venba", significantly narrowing down the classification tree for the higher layers.



In the Thalai Layer (Linkage) and Adi Layer (Line Structure), the rule engine moves from looking at individual words to looking at the "flow" and "geometry" of the poem.
1. Thalai Layer (Linkage Level)
This layer calculates how sorkal[i] (the "standing foot" or Nindra Seer) connects to sorkal[i+1] (the "coming foot" or Varum Seer).

| Field | Logic / Calculation | Possible Values |
|---|---|---|
| link_type | Compare the Seer Group of Word[n] with the First Asai of Word[n+1]. | Iyarseer Ventalai, Kalithalai, etc. |
| is_ventalai | Boolean: Is the link either Iyarseer Ventalai or Venseer Ventalai? | True / False |
| link_harmony | Checks if the link matches the preceding link (used for rhythm consistency). | Consistent / Variable |

The Thalai Logic Table (The "Engine Room"):
The rule engine uses this mapping to derive link_type:

* Ma-seer + Nirai $\rightarrow$ Iyarseer Ventalai
* Vila-seer + Ner $\rightarrow$ Iyarseer Ventalai
* Kaai-seer + Ner $\rightarrow$ Venseer Ventalai
* Kaai-seer + Nirai $\rightarrow$ Kalithalai
* Kani-seer + Nirai $\rightarrow$ Oonru Vanjithalai
* Kani-seer + Ner $\rightarrow$ Oonraatha Vanjithalai
* Ma-seer + Ner $\rightarrow$ Ner-onru Aasiriyathalai
* Vila-seer + Nirai $\rightarrow$ Nirai-onru Aasiriyathalai

------------------------------
2. Adi Layer (Line Structure Level)
This layer looks at the line as a whole. It derives fields based on word counts and position.

| Field | Logic / Calculation | Possible Values |
|---|---|---|
| word_count | Length of the sorkal array for that specific line. | 2, 3, 4, 5+ |
| adi_type | Categorization based on word_count. | Kuraladi (2), Chinthadi (3), Alavadi (4), Neduladi (5), Kazhineduladi (6+) |
| is_standard_venba_line | Boolean: Is adi_type == Alavadi AND all links in the line is_ventalai? | True / False |
| has_thanichol | Check if the 4th foot is followed by a special "break" or dash (specific to Nerisai Venba). | True / False |
| line_position | Index-based tag. | First, Middle, Penultimate, Last |

Key Derived Logic for Classification:

* Penultimate Check: If line_position == Penultimate AND adi_type == Chinthadi, the engine flags a potential Nerisai sub-type (for both Venba and Asiriyappa).
* Final Line Check: For Venba, the last line must be Chinthadi (3 words).

------------------------------
Summary of what you have now:

   1. Seer Layer: Knows what the words are (e.g., "This is a Ma-seer").
   2. Thalai Layer: Knows how they shake hands (e.g., "This is a Ventalai connection").
   3. Adi Layer: Knows the shape of the line (e.g., "This is a 4-word line").



   In these final layers, the rule engine moves from "how a line is built" to "how the entire poem relates to itself." This is where you finalize the specific sub-classification.
   4. Thodai Layer (Rhyme & Pattern Level)
   This layer looks across the lines to identify rhyme consistency and rhythmic ornaments.
   
   | Field | Logic / Calculation | Possible Values |
   |---|---|---|
   | rhyme_id_list | A collection of irandaam_ezhuthu from the first word of every line. | ['da', 'da', 'ka', 'ka'] |
   | vikarpam_count | The number of unique rhyme_id groups. | 1, 2, 3, 4 |
   | vikarpam_type | Categorization based on vikarpam_count. | Oru-vikarpam (1), Iru-vikarpam (2), Pala-vikarpam (>2) |
   | is_iyaipu | True if kadai_ezhuthu matches across all or most lines. | Boolean |
   | rhyme_schema | Pattern of the rhyme (e.g., 1-2 match, 3-4 match). | AABB, ABAB, AAAA |
   
   Sub-classification Logic for Venba:
   
   * Nerisai Venba: Requires a thanichol (standalone word) at the end of the 2nd line AND usually an Iru-vikarpam (2+2) or Oru-vikarpam rhyme scheme.
   * Innisai Venba: No thanichol at the 2nd line; often uses Pala-vikarpam.
   
   ------------------------------
   5. Final Pa Layer (Classification Level)
   This is the output layer. It aggregates all previous flags into the final verdict.
   
   | Field | Logic / Calculation | Possible Values |
   |---|---|---|
   | primary_pa | Derived from the dominant link_type and adi_type. | Venba, Asiriyappa, Kalippa, Vanjippa |
   | osai_type | The "sound" assigned based on primary_pa. | Seppal, Ahaval, Thullal, Thoongal |
   | granularity_type | The specific sub-type based on line count and geometry. | See list below |
   | is_valid | Safety check: Does it follow the mandatory rules (e.g., Venba ending in Naal/Malar)? | True / False |
   
   The "Granularity" Logic Tree:
   
      1. If Venba (all links are Ventalai):
      * 2 lines $\rightarrow$ Kural Venba
         * 3 lines $\rightarrow$ Sindhial Venba
         * 4 lines + Thanichol $\rightarrow$ Nerisai Venba
         * 4 lines + No Thanichol $\rightarrow$ Innisai Venba
         * 13+ lines $\rightarrow$ Pahrodai Venba
      2. If Asiriyappa (dominant Aasiriyathalai + ends in 'ஏ'):
      * Penultimate line is 3 words $\rightarrow$ Nerisai Asiriyappa
         * First and last lines are 4 words, middle are 2/3 $\rightarrow$ Inaikural Asiriyappa
         * All lines are 4 words $\rightarrow$ Nilaimandila Asiriyappa
      3. If Vanjippa (dominant Vanjithalai):
      * Lines of 2 words $\rightarrow$ Kuraladi Vanjippa
         * Lines of 3 words $\rightarrow$ Chinthadi Vanjippa
      
   Final Tip for the Rule Engine:
   Always run the Venba check first. It is the most "strict" (mathematical) classification. If even one link fails is_ventalai, move to Asiriyappa, which is the most "flexible."
