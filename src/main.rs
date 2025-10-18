use chrono::{Datelike, Duration, TimeZone, Utc};
use lindera::{dictionary::load_dictionary, mode::Mode, segmenter::Segmenter};
use lindera_tantivy::tokenizer::LinderaTokenizer;
use tantivy::{
    Document, Index, Order, TantivyDocument, Term,
    collector::TopDocs,
    doc,
    query::{BooleanQuery, Occur, Query, QueryParser, TermQuery},
    schema::{FAST, INDEXED, IndexRecordOption, STORED, Schema, TextFieldIndexing, TextOptions},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create schema builder
    let mut schema_builder = Schema::builder();

    // add id field
    let id = schema_builder.add_text_field(
        "id",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("raw")
                    .set_index_option(IndexRecordOption::Basic),
            )
            .set_stored()
            .set_fast(Some("raw")),
    );

    // add date field
    let date = schema_builder.add_date_field("date", STORED | INDEXED | FAST);

    // add draft field
    let draft = schema_builder.add_bool_field("draft", STORED | INDEXED);

    // add title field
    let title = schema_builder.add_text_field(
        "title",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("lang_ja")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored(),
    );

    // add body field
    let body = schema_builder.add_text_field(
        "body",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("lang_ja")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored(),
    );

    // add tags field
    let tags = schema_builder.add_text_field(
        "tags",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("raw")
                    .set_index_option(IndexRecordOption::Basic),
            )
            .set_stored(),
    );

    // build schema
    let schema = schema_builder.build();

    // create index on memory
    let index = Index::create_in_ram(schema.clone());

    // Tokenizer with IPADIC
    let mode = Mode::Normal;
    let dictionary = load_dictionary("./mecab-ipadic")?;
    let user_dictionary = None;
    let segmenter = Segmenter::new(mode, dictionary, user_dictionary);
    let tokenizer = LinderaTokenizer::from_segmenter(segmenter);

    // register Lindera tokenizer
    index.tokenizers().register("lang_ja", tokenizer);

    // create index writer
    let mut index_writer = index.writer(50_000_000)?;

    // add document
    let index_date = tantivy::DateTime::from_timestamp_secs(Utc::now().timestamp());
    let mut doc = doc!(
        id => "1",
        title => "成田国際空港",
        date => index_date,
        draft => false,
        body => "成田国際空港（なりたこくさいくうこう、英: Narita International Airport）は、千葉県成田市南東部から芝山町北部にかけて建設された日本最大の国際拠点空港である。首都圏東部（東京の東60km）に位置している。空港コードはNRT。"
    );
    doc.add_text(tags, "空港");
    doc.add_text(tags, "国際");
    doc.add_text(tags, "成田");
    index_writer.add_document(doc)?;

    // add document
    let index_date =
        tantivy::DateTime::from_timestamp_secs((Utc::now() + Duration::days(-1)).timestamp());
    let mut doc = doc!(
        id => "2",
        title => "東京国際空港",
        date => index_date,
        draft => false,
        body => "東京国際空港（とうきょうこくさいくうこう、英語: Tokyo International Airport）は、東京都大田区にある日本最大の空港。通称は羽田空港（はねだくうこう、英語: Haneda Airport）であり、単に「羽田」と呼ばれる場合もある。空港コードはHND。"
    );
    doc.add_text(tags, "空港");
    doc.add_text(tags, "羽田");
    doc.add_text(tags, "東京");
    index_writer.add_document(doc)?;

    // add document
    let index_date =
        tantivy::DateTime::from_timestamp_secs((Utc::now() + Duration::days(-2)).timestamp());
    let mut doc = doc!(
        id => "3",
        title => "関西国際空港",
        date => index_date,
        draft => false,
        body => "関西国際空港（かんさいこくさいくうこう、英: Kansai International Airport）は大阪市の南西35㎞に位置する西日本の国際的な玄関口であり、関西三空港の一つとして大阪国際空港（伊丹空港）、神戸空港とともに関西エアポート株式会社によって一体運営が行われている。"
    );
    doc.add_text(tags, "空港");
    doc.add_text(tags, "国際");
    doc.add_text(tags, "関西");
    index_writer.add_document(doc)?;

    for i in 0..20 {
        let idx = i + 4;
        let index_date = tantivy::DateTime::from_timestamp_secs(
            (Utc::now() + Duration::days(idx * -1)).timestamp(),
        );

        let mut doc = doc!(
            id => format!("{}", idx),
            title => format!("ダミータイトル {}", idx),
            date => index_date,
            draft => true,
            body => format!("ダミーテキスト {}", idx),
        );
        doc.add_text(tags, "空港");
        index_writer.add_document(doc)?;
    }

    // commit
    index_writer.commit()?;

    // create reader
    let reader = index.reader()?;

    // create searcher
    let searcher = reader.searcher();

    {
        // 短項目全文検索(空白で区切ってOR)
        // parse query
        let query_str = "東京 成田 ダミー";
        // create querhy parser
        let query_parser = QueryParser::for_index(&index, vec![title, body]);
        let query = query_parser.parse_query(query_str)?;
        println!("Query String: {}", query_str);

        // search
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        println!("Search Result:");
        for (_, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("{}", retrieved_doc.to_json(&schema));
        }
    }

    {
        // 日付範囲検索
        println!();
        let now = Utc::now();
        let start_date = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .single()
            .ok_or("invalid datetime")?;
        let end_date = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 23, 59, 59)
            .single()
            .ok_or("invalid datetime")?;
        let q = format!(
            "date:[{} TO {}]",
            start_date.to_rfc3339(),
            end_date.to_rfc3339()
        );
        println!("Query String: {}", q);

        let query_parser = QueryParser::for_index(&index, vec![date]);
        let query = query_parser.parse_query(&q)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        println!("Search Result:");
        for (_, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("{}", retrieved_doc.to_json(&schema));
        }
    }

    {
        // 日付範囲と短項目全文検索の組み合わせ(AND)
        println!();
        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        let yestaday = Utc::now() + Duration::days(-1);
        let start_date = Utc
            .with_ymd_and_hms(yestaday.year(), yestaday.month(), yestaday.day(), 0, 0, 0)
            .single()
            .ok_or("invalid datetime")?;
        let end_date = Utc
            .with_ymd_and_hms(
                yestaday.year(),
                yestaday.month(),
                yestaday.day(),
                23,
                59,
                59,
            )
            .single()
            .ok_or("invalid datetime")?;
        let q = format!(
            "date:[{} TO {}]",
            start_date.to_rfc3339(),
            end_date.to_rfc3339()
        );
        println!("Query String: {}", q);

        let query_parser = QueryParser::for_index(&index, vec![date]);
        let query = query_parser.parse_query(&q)?;
        queries.push((Occur::Must, query));

        let query_str = "東京";
        println!("Query String: {}", query_str);
        let query_parser = QueryParser::for_index(&index, vec![title, body]);
        let query = query_parser.parse_query(query_str)?;
        queries.push((Occur::Must, query));

        let query = BooleanQuery::from(queries);
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        println!("Search Result:");
        for (_, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("{}", retrieved_doc.to_json(&schema));
        }
    }

    {
        // 日付範囲のページネーション
        println!();
        let start_date = Utc
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .single()
            .ok_or("invalid datetime")?;
        // *は無限大を表す
        let q = format!("date:[{} TO *]", start_date.to_rfc3339(),);
        println!("Query String: {}", q);
        let query_parser = QueryParser::for_index(&index, vec![date]);
        let query = query_parser.parse_query(&q)?;

        let collector = TopDocs::with_limit(5)
            .and_offset(0)
            .order_by_fast_field::<tantivy::DateTime>("date", Order::Desc);
        let top_docs = searcher.search(&query, &collector)?;
        println!("Search Result:");
        for (_, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("{}", retrieved_doc.to_json(&schema));
        }
    }

    {
        // 完全一致(OR)
        println!();
        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        let tags_value = vec!["羽田"];

        let tag_queries: Vec<Box<dyn Query>> = tags_value
            .iter()
            .map(|tag| tag.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                // ↓ Term::from_facet から Term::from_field_text に戻します
                let term = Term::from_field_text(tags, s);
                Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>
            })
            .collect();

        let mut tags_or_query = BooleanQuery::from(
            tag_queries
                .into_iter()
                .map(|q| (Occur::Should, q))
                .collect::<Vec<_>>(),
        );
        tags_or_query.set_minimum_number_should_match(1);
        queries.push((Occur::Must, Box::new(tags_or_query)));

        let term = Term::from_field_bool(draft, false);
        let draft_query = TermQuery::new(term, IndexRecordOption::Basic);
        queries.push((Occur::Must, Box::new(draft_query)));

        let query = BooleanQuery::from(queries);
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        println!("Search Result:");
        for (_, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("{}", retrieved_doc.to_json(&schema));
        }
    }

    {
        // 完全一致
        println!();
        let term = Term::from_field_text(id, "1");
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        let top_docs = searcher.search(&query, &TopDocs::with_limit(2))?;
        println!("Search Result:");
        for (_, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("{}", retrieved_doc.to_json(&schema));
        }
    }

    Ok(())
}
