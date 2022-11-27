use rstest::rstest;

#[rstest]
#[case("", "", "")]
#[case("構成", "こうせい", "kousei")]
#[case("好き", "すき", "suki")]
#[case("大きい", "おおきい", "ookii")]
#[case("かんたん", "かんたん", "kantan")]
#[case("にゃ", "にゃ", "nya")]
#[case("っき", "っき", "kki")]
#[case("っふぁ", "っふぁ", "ffa")]
#[case("キャ", "きゃ", "kya")]
#[case("キュ", "きゅ", "kyu")]
#[case("キョ", "きょ", "kyo")]
#[case("。", "。", ".")]
#[case(
    "漢字とひらがな交じり文",
    "かんじとひらがなまじりぶん",
    "kanji tohiragana majiri bun"
)]
#[case(
    "Alphabet 123 and 漢字",
    "Alphabet 123 and かんじ",
    "Alphabet 123 and kanji"
)]
#[case("日経新聞", "にっけいしんぶん", "nikkei shinbun")]
#[case("日本国民は、", "にほんこくみんは、", "nihonkokumin ha,")]
#[case(
    "私がこの子を助けなきゃいけないってことだよね",
    "わたしがこのこをたすけなきゃいけないってことだよね",
    "watashi gakono ko wo tasuke nakyaikenaittekotodayone"
)]
#[case("やったー", "やったー", "yattaa")]
#[case("でっでー", "でっでー", "deddee")]
#[case("てんさーふろー", "てんさーふろー", "tensaafuroo")]
#[case("オレンジ色", "おれんじいろ", "orenji iro")]
#[case("檸檬は、レモン色", "れもんは、れもんいろ", "remon ha, remon iro")]
#[case("血液1μL", "けつえき1μL", "ketsueki 1μL")]
#[case("「和風」", "「わふう」", "\"wafuu\"")]
#[case("て「わ", "て「わ", "te \"wa")]
#[case("号・雅", "ごう・まさ", "gou masa")]
#[case("ビーバーが", "びーばーが", "biibaa ga")]
#[case("ブッシュッー", "ぶっしゅっー", "busshutsuu")]
#[case("ユーベルヹーク大", "ゆーべるゔぇーくだい", "yuuberuveeku dai")]
#[case("ヸーヂャニー品", "ゔぃーぢゃにーひん", "viijanii hin")]
#[case("アヷーリヤ品", "あゔぁーりやひん", "avaariya hin")]
#[case(
        "安藤 和風（あんどう はるかぜ、慶応2年1月12日（1866年2月26日） - 昭和11年（1936年）12月26日）は、日本のジャーナリスト、マスメディア経営者、俳人、郷土史研究家。通名および俳号は「和風」をそのまま音読みして「わふう」。秋田県の地方紙「秋田魁新報」の事業拡大に貢献し、秋田魁新報社三大柱石の一人と称された。「魁の安藤か、安藤の魁か」と言われるほど、新聞記者としての名声を全国にとどろかせた[4]。",
        "あんどう わふう（あんどう はるかぜ、けいおう2ねん1がつ12にち（1866ねん2がつ26にち） - しょうわ11ねん（1936ねん）12がつ26にち）は、にっぽんのじゃーなりすと、ますめでぃあけいえいしゃ、はいじん、きょうどしけんきゅうか。とおりめいおよびはいごうは「わふう」をそのままおんよみして「わふう」。あきたけんのちほうし「あきたかいしんぽう」のじぎょうかくだいにこうけんし、あきたかいしんぽうしゃさんだいちゅうせきのひとりとしょうされた。「かいのあんどうか、あんどうのかいか」といわれるほど、しんぶんきしゃとしてのめいせいをぜんこくにとどろかせた[4]。",
        "Andou wafuu (andou harukaze, keiou 2 nen 1 gatsu 12 nichi (1866 nen 2 gatsu 26 nichi) - shouwa 11 nen (1936 nen) 12 gatsu 26 nichi) ha, nippon no jaanarisuto, masumedia keieisha, haijin, kyoudoshi kenkyuuka. Toori mei oyobi hai gou ha \"wafuu\" wosonomama onyomi shite \"wafuu\". Akitaken no chihoushi \"akita kai shinpou\" no jigyou kakudai ni kouken shi, akita kai shinpou sha sandai chuuseki no hitori to shousa reta. \"Kai no andou ka, andou no kai ka\" to iwa reruhodo, shinbunkisha toshiteno meisei wo zenkoku nitodorokaseta [4].",
    )]
#[case(
    "『ザ・トラベルナース』",
    "『ざ・とらべるなーす』",
    "\"za toraberunaasu\""
)]
#[case(
    "緑黄色社会『ミチヲユケ』Official Video -「ファーストペンギン！」主題歌",
    "みどりきいろしゃかい『みちをゆけ』Official Video -「ふぁーすとぺんぎん！」しゅだいか",
    "midori kiiro shakai \"michiwoyuke\" Official Video - \"faasutopengin!\" shudaika"
)]
#[case(
    "MONKEY MAJIK - Running In The Dark【Lyric Video】（日本語字幕付）",
    "MONKEY MAJIK - Running In The Dark【Lyric Video】（にほんごじまくつき）",
    "MONKEY MAJIK - Running In The Dark [Lyric Video] (nihongo jimaku tsuki)"
)]
#[case(
    "取締役第二制作技術部々長",
    "とりしまりやくだいにせいさくぎじゅつぶぶちょう",
    "torishimariyaku daini seisaku gijutsubu buchou"
)]
#[case(
    "最初の安定版である1.0版がリリ",
    "さいしょのあんていはんである1.0はんがりり",
    "saisho no antei han dearu 1.0 han ga riri"
)]
#[case("にゃ＄にゃ", "にゃ＄にゃ", "nya $ nya")]
#[case(
        "安定版となるRust 1.0がリリースされた[84]。1.0版の後、安定版およびベータ版が6週間おきに定期リリースされている[85]。",
        "あんていはんとなるRust 1.0がりりーすされた[84]。1.0はんののち、あんていはんおよびべーたはんが6しゅうかんおきにていきりりーすされている[85]。",
        "Antei han tonaru Rust 1.0 ga ririisu sareta [84]. 1.0 han no nochi, antei han oyobi beeta han ga 6 shuukan okini teiki ririisu sareteiru [85]."
    )]
#[case(
    "prelude文にTryIntoやTryFrom",
    "preludeぶんにTryIntoやTryFrom",
    "prelude bun ni TryInto ya TryFrom"
)]
#[case("要所々々", "ようしょようしょ", "yousho yousho")]
#[case("Hello World. abcd.", "Hello World. abcd.", "Hello World. abcd.")]
fn romanize(#[case] text: &str, #[case] hiragana: &str, #[case] romaji: &str) {
    let res = kakasi::convert(text);
    assert_eq!(res.hiragana, hiragana);
    assert_eq!(res.romaji, romaji);
}
