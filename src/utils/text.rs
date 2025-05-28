use std::{cmp::Ordering, hash::BuildHasherDefault, sync::LazyLock};

use fnv::FnvHasher;
use jieba_rs::{Jieba, KeywordExtract, KeywordExtractConfig, TfIdf};
use pinyin::ToPinyin;
use probminhash::densminhash::RevOptDensMinHash;
use regex::Regex;
use text_splitter::{ChunkConfig, MarkdownSplitter, TextSplitter};
use tiktoken_rs::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    TRUNCATE_LOG_MESSAGE,
    error::{AiterError, AiterResult},
};

#[derive(strum::EnumString, strum::Display, Copy, Clone)]
#[strum(ascii_case_insensitive)]
pub enum Tokenizer {
    O200kBase,
}

pub fn compare_phonetic(a: &str, b: &str) -> Ordering {
    natord::compare(&to_phonetic(a), &to_phonetic(b))
}

pub fn minhash(text: &str, num_hashes: usize, tokenizer: &Tokenizer) -> AiterResult<Vec<f32>> {
    let tokens = to_tokens(text, tokenizer);
    if tokens.is_empty() {
        return Err(AiterError::HashError(format!("Failed to hash: {text}")));
    }

    let build_hasher = BuildHasherDefault::<FnvHasher>::default();
    let mut minhash: RevOptDensMinHash<f32, u32, FnvHasher> =
        RevOptDensMinHash::new(num_hashes, build_hasher);

    minhash.sketch_slice(&tokens).map_err(|_| {
        AiterError::HashError(format!(
            "Failed to hash: {}",
            truncate_format(text, TRUNCATE_LOG_MESSAGE, false)
        ))
    })?;
    Ok(minhash.get_hsketch().to_vec())
}

pub fn split_by_max_tokens(text: &str, max_tokens: usize, tokenizer: &Tokenizer) -> Vec<String> {
    let bpe = match tokenizer {
        Tokenizer::O200kBase => o200k_base_singleton(),
    };

    let splitter = TextSplitter::new(ChunkConfig::new(max_tokens).with_sizer(bpe));
    splitter.chunks(text).map(|s| s.to_string()).collect()
}

pub fn split_markdown_by_max_tokens(
    text: &str,
    max_tokens: usize,
    tokenizer: &Tokenizer,
) -> Vec<String> {
    let bpe = match tokenizer {
        Tokenizer::O200kBase => o200k_base_singleton(),
    };

    let splitter = MarkdownSplitter::new(ChunkConfig::new(max_tokens).with_sizer(bpe));
    splitter.chunks(text).map(|s| s.to_string()).collect()
}

pub fn to_tokens(text: &str, tokenizer: &Tokenizer) -> Vec<u32> {
    match tokenizer {
        Tokenizer::O200kBase => {
            let bpe = o200k_base_singleton();
            bpe.encode_ordinary(text)
        }
    }
}

pub fn to_words(text: &str, extract_keywords: bool) -> Vec<String> {
    let words = if extract_keywords {
        extract_key_words(text)
            .into_iter()
            .flat_map(|s| split_words(&s))
            .collect()
    } else {
        split_words(text)
    };

    words
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && !REGEX_WORD_FILTER.is_match(s))
        .collect()
}

pub fn truncate_format(text: &str, max_chars: usize, replace_lf: bool) -> String {
    let (s, t) = truncate(text, max_chars);
    let s = if t { s + "..." } else { s };

    if replace_lf { s.replace("\n", " ") } else { s }
}

static JIEBA: LazyLock<Jieba> = LazyLock::new(Jieba::new);
static JIEBA_TFIDF: LazyLock<TfIdf> = LazyLock::new(|| {
    let config = KeywordExtractConfig::builder()
        .use_hmm(true)
        .build()
        .unwrap();

    TfIdf::new(None::<&mut std::io::Empty>, config)
});
static REGEX_WORD_FILTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\p{C}\p{P}\p{Z}\p{Extended_Pictographic}\u{FE0F}<>]")
        .expect("WORD_FILTER regex is invalid")
});

fn extract_key_words(text: &str) -> Vec<String> {
    (*JIEBA_TFIDF)
        .extract_keywords(&JIEBA, text, text.len(), vec![])
        .iter()
        .map(|k| k.keyword.to_string())
        .collect()
}

fn is_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{3400}'..='\u{9FFF}' |
            '\u{20000}'..='\u{2EE5D}' |
            '\u{30000}'..='\u{323AF}' |
            '\u{3040}'..='\u{30FF}' | // Japanese
            '\u{AC00}'..='\u{D7FF}' // Korean
        )
    })
}

fn split_words(text: &str) -> Vec<String> {
    let mut words: Vec<String> = vec![];

    let mut current_word = String::new();
    for grapheme in text.graphemes(true) {
        if is_cjk(grapheme) {
            if !current_word.is_empty() {
                words.push(current_word.clone());
                current_word.clear();
            }
            words.push(grapheme.to_string());
        } else if grapheme.chars().all(|c| c.is_alphabetic()) {
            current_word.push_str(grapheme);
        } else {
            if !current_word.is_empty() {
                words.push(current_word.clone());
                current_word.clear();
            }
            words.push(grapheme.to_string());
        }
    }

    if !current_word.is_empty() {
        words.push(current_word);
    }

    words
}

fn to_phonetic(text: &str) -> String {
    let mut s = String::new();

    text.chars().for_each(|c| {
        if matches!(
            c,
            '\u{3400}'..='\u{9FFF}' |
            '\u{20000}'..='\u{2EE5D}' |
            '\u{30000}'..='\u{323AF}'
        ) {
            let pinyin = c
                .to_pinyin()
                .iter()
                .map(|p| p.with_tone_num())
                .collect::<Vec<_>>()
                .join(" ");
            s.push_str(&pinyin);
        } else {
            s.push(c);
        }
    });

    s
}

fn truncate(text: &str, max_chars: usize) -> (String, bool) {
    if text.chars().count() > max_chars {
        let mut end = 0;
        for (i, (idx, _)) in text.char_indices().enumerate() {
            if i == max_chars {
                end = idx;
                break;
            }
        }

        (text[..end].to_string(), true)
    } else {
        (text.to_string(), false)
    }
}

#[cfg(test)]
mod tests {
    use probminhash::jaccard::compute_probminhash_jaccard;

    use super::*;
    use crate::{CURRENT_SIGNATURE_DIMS, CURRENT_TOKENIZER};

    #[test]
    fn test_compare_phonetic() {
        let mut texts = ["你好", "世界", "こんにち", "저는", "Hello", "world"];
        texts.sort_by(|a, b| compare_phonetic(&a, &b));

        assert_eq!(
            texts,
            ["Hello", "你好", "世界", "world", "こんにち", "저는"]
        );
    }

    #[test]
    fn test_minhash() {
        let v1 = minhash(
            "Hello world, I am aiter. 你好こんにちは저는",
            CURRENT_SIGNATURE_DIMS,
            &CURRENT_TOKENIZER,
        )
        .unwrap();
        let v2 = minhash(
            "Hello world, I am aiter",
            CURRENT_SIGNATURE_DIMS,
            &CURRENT_TOKENIZER,
        )
        .unwrap();
        let v3 = minhash(
            "Hello world, aiter.",
            CURRENT_SIGNATURE_DIMS,
            &CURRENT_TOKENIZER,
        )
        .unwrap();

        assert_eq!(v1.len(), CURRENT_SIGNATURE_DIMS);

        let dis_v1_v2 = v1
            .iter()
            .zip(v2.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();
        let dis_v1_v3 = v1
            .iter()
            .zip(v3.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();
        assert!(dis_v1_v2 < dis_v1_v3);
    }

    #[test]
    fn test_similarity() {
        let v0 = minhash(
            "瑞幸咖啡有多少门店",
            CURRENT_SIGNATURE_DIMS,
            &CURRENT_TOKENIZER,
        )
        .unwrap();
        let v1 = minhash(
            "瑞幸咖啡的门店数",
            CURRENT_SIGNATURE_DIMS,
            &CURRENT_TOKENIZER,
        )
        .unwrap();
        let v2 = minhash(
            "和幸咖啡的门店数",
            CURRENT_SIGNATURE_DIMS,
            &CURRENT_TOKENIZER,
        )
        .unwrap();

        let v0_v1 = compute_probminhash_jaccard(&v0, &v1);
        let v0_v2 = compute_probminhash_jaccard(&v0, &v2);
        assert!(v0_v1 > v0_v2);
    }

    #[test]
    fn test_split_by_max_tokens() {
        let text = r"
列位看官：你道此书从何而来？说起根由，虽近荒唐，细按则深有趣味。待在下将此来历注明，方使阅者了然不惑。

原来女娲氏炼石补天之时，于大荒山无稽崖炼成高经十二丈、方经二十四丈顽石三万六千五百零一块。娲皇氏只用了三万六千五百块，只单单剩了一块未用，便弃在此山青埂峰下。谁知此石自经煅炼之后，灵性已通，因见众石俱得补天，独自己无材不堪入选，遂自怨自叹，日夜悲号惭愧。

一日，正当嗟悼之际，俄见一僧一道远远而来，生得骨格不凡，丰神迥别，说说笑笑，来至峰下，坐于石边，高谈快论：先是说些云山雾海、神仙玄幻之事，后便说到红尘中荣华富贵。此石听了，不觉打动凡心，也想要到人间去享一享这荣华富贵，但自恨粗蠢，不得已，便口吐人言，向那僧道说道：“大师，弟子蠢物，不能见礼了！适闻二位谈那人世间荣耀繁华，心切慕之。弟子质虽粗蠢，性却稍通，况见二师仙形道体，定非凡品，必有补天济世之材，利物济人之德。如蒙发一点慈心，携带弟子得入红尘，在那富贵场中，温柔乡里受享几年，自当永佩洪恩，万劫不忘也！”二仙师听毕，齐憨笑道：“善哉，善哉！那红尘中有却有些乐事，但不能永远依恃；况又有‘美中不足，好事多磨’八个字紧相连属，瞬息间则又乐极悲生，人非物换，究竟是到头一梦，万境归空，倒不如不去的好。”这石凡心已炽，那里听得进这话去，乃复苦求再四。二仙知不可强制，乃叹道：“此亦静极思动，无中生有之数也！既如此，我们便携你去受享受享，只是到不得意时，切莫后悔！”石道：“自然，自然。”那僧又道：“若说你性灵，却又如此质蠢，并更无奇贵之处。如此也只好踮脚而已。也罢！我如今大施佛法，助你助，待劫终之日，复还本质，以了此案。你道好否？”石头听了，感谢不尽。那僧便念咒书符，大展幻术，将一块大石登时变成一块鲜明莹洁的美玉，且又缩成扇坠大小的可佩可拿。那僧托于掌上，笑道：“形体倒也是个宝物了！还只没有实在的好处，须得再镌上数字，使人一见便知是奇物方妙。然后好携你到那昌明隆盛之邦、诗礼簪缨之族、花柳繁华地、温柔富贵乡去安身乐业。”石头听了，喜不能禁，乃问：“不知赐了弟子那哪几件奇处？又不知携了弟子到何地方？望乞明示，使弟子不惑。”那僧笑道：“你且莫问，日后自然明白的。”说着，便袖了这石，同那道人飘然而去，竟不知投奔何方何舍。

后来，不知过了几世几劫，因有个空空道人访道求仙，从这大荒山无稽崖青埂峰下经过，忽见一大块石上字迹分明，编述历历。空空道人乃从头一看，原来就是无材补天，幻形入世，蒙茫茫大士、渺渺真人携入红尘，历尽离合悲欢、炎凉世态的一段故事。后面又有一首偈云：

无材可去补苍天，枉入红尘若许年。此系身前身后事，倩谁记去作奇传？

诗后便是此石坠落之乡，投胎之处，亲自经历的一段陈迹故事。其中家庭闺阁琐事，以及闲情诗词倒还全备，或可适趣解闷；然朝代年纪、地舆邦国却反失落无考。

空空道人遂向石头说道：“石兄，你这一段故事，据你自己说有些趣味，故编写在此，意欲问世传奇。据我看来：第一件，无朝代年纪可考；第二件，并无大贤大忠理朝廷、治风俗的善政，其中只不过几个异样女子，或情或痴，或小才微善，亦无班姑、蔡女之德能。我纵抄去，恐世人不爱看呢！”石头笑答道：“我师何太痴耶！若云无朝代可考，今我师竟借汉、唐等年纪添缀，又有何难？但我想，历来野史，皆蹈一辙，莫如我这不借此套者，反倒新奇别致。不过只取其事体情理罢了，又何必拘拘于朝代年纪哉！再者，市井俗人喜看理治之书者甚少，爱适趣闲文者特多。历来野史，或讪谤君相，或贬人妻女，奸淫凶恶，不可胜数。更有一种风月笔墨，其淫秽污臭，屠毒笔墨，坏人子弟，又不可胜数。至若佳人才子等书，则又千部共出一套，且其中终不能不涉于淫滥，以致满纸潘安、子建、西子、文君。不过作者要写出自己的那两首情诗艳赋来，故假拟出男女二人名姓，又必旁出一小人其间拨乱，亦如剧中之小丑然。且鬟婢开口即者也之乎，非文即理。故逐一看去，悉皆自相矛盾、大不近情理之话，竟不如我半世亲睹亲闻的这几个女子，虽不敢说强似前代书中所有之人，但事迹原委，亦可以消愁破闷；也有几首歪诗熟话，可以喷饭供酒。至若离合悲欢，兴衰际遇，则又追踪蹑迹，不敢稍加穿凿，徒为供人之目而反失其真传者。今之人，贫者日为衣食所累，富者又怀不足之心；纵然一时稍闲，又有贪淫恋色、好货寻愁之事，哪里有工夫去看那理治之书！所以，我这一段故事，也不愿世人称奇道妙，也不定要世人喜悦检读，只愿他们当那醉淫饱卧之时，或避世去愁之际，把此一玩，岂不省了些寿命筋力？就比那谋虚逐妄，却也省了口舌是非之害、腿脚奔忙之苦。再者，亦令世人换新眼目，不比那些胡牵乱扯，忽离忽遇，满纸才人淑女、子建、文君、红娘、小玉等通共熟套之旧稿。我师意为何如？”

空空道人听如此说，思忖半晌，将一这《石头记》再检阅一遍，因见上面虽有些指奸责佞、贬恶诛邪之语，亦非伤时骂世之旨；及至君仁臣良、父慈子孝，凡伦常所关之处，皆是称功颂德，眷眷无穷，实非别书之可比。虽其中大旨谈情，亦不过实录其事，又非假拟妄称，一味淫邀艳约，私订偷盟之可比。因毫不干涉时世，方从头至尾抄录回来，问世传奇。因空见色，由色生情，传情入色，自色悟空，空空道人遂易名为情僧，改《石头记》为《情僧录》。至?玉峰题曰《红楼梦》。东鲁孔梅溪则题曰《风月宝鉴》。后因曹雪芹于悼红轩中，披阅十载，增删五次，纂成目录，分出章回，则题曰《金陵十二钗》，并题一绝云：

满纸荒唐言，一把辛酸泪！都云作者痴，谁解其中味？

至脂砚斋甲戌抄阅再评，仍用《石头记》。
        ";

        for n in [10, 100, 1000] {
            let split = split_by_max_tokens(text, n, &CURRENT_TOKENIZER);

            for s in &split {
                assert!(to_tokens(s, &CURRENT_TOKENIZER).len() <= n);
            }

            assert_eq!(
                split
                    .iter()
                    .map(|s| s.chars().filter(|s| *s != '\n' && *s != ' ').count())
                    .sum::<usize>(),
                text.chars().filter(|s| *s != '\n' && *s != ' ').count()
            );
        }

        assert_eq!(split_by_max_tokens(text, 4000, &CURRENT_TOKENIZER).len(), 1);
    }

    #[test]
    fn test_to_tokens() {
        let text = r#"Aiter said: "Hello world.". 我说：“你好！”。完美的一天：)"#;
        let tokens = to_tokens(text, &CURRENT_TOKENIZER);
        assert_eq!(o200k_base().unwrap().decode(tokens).unwrap(), text);
    }

    #[test]
    fn test_to_words() {
        let words = to_words(
            r#"Aiter said: "Hello world.". 我说：“你好！”。完美的一天：) ><"#,
            false,
        );
        assert_eq!(
            words,
            vec![
                "aiter", "said", "hello", "world", "我", "说", "你", "好", "完", "美", "的", "一",
                "天"
            ]
        );
    }

    #[test]
    fn test_to_words_extract_keywords() {
        let words = to_words(r#"Hello, 和幸的门店"#, true);
        assert!(words.contains(&"hello".to_string()));
        assert!(!words.contains(&"和".to_string())); // Some words may not be extracted
        assert!(!words.contains(&"幸".to_string()));
        assert!(!words.contains(&"的".to_string()));
        assert!(words.contains(&"门".to_string()));
        assert!(words.contains(&"店".to_string()));
    }
}
