use shinoho_anime::*;

const WATCHING: &str = r#"{"data":{"MediaListCollection":{"lists":[
  {"entries":[
    {"progress":3,"status":"CURRENT","media":{"id":1,"episodes":12,"siteUrl":"https://anilist.co/anime/1",
      "title":{"romaji":"Sousou no Frieren","english":"Frieren: Beyond Journey's End","native":"葬送のフリーレン"},
      "coverImage":{"large":"https://img/1.jpg"},
      "nextAiringEpisode":{"episode":6,"airingAt":1791900000}}},
    {"progress":24,"status":"CURRENT","media":{"id":2,"episodes":24,"siteUrl":null,
      "title":{"romaji":"Kusuriya no Hitorigoto","english":null,"native":"薬屋のひとりごと"},
      "coverImage":{"large":null},"nextAiringEpisode":null}}
  ]},
  {"entries":[
    {"progress":10,"status":"REPEATING","media":{"id":3,"episodes":null,"siteUrl":null,
      "title":{"romaji":"One Piece","english":"ONE PIECE","native":null},
      "coverImage":{"large":null},"nextAiringEpisode":{"episode":12,"airingAt":1791990000}}}
  ]}
]}}}"#;

#[test]
fn login_url_and_redirect() {
    assert_eq!(
        authorize_url("12345"),
        "https://anilist.co/api/v2/oauth/authorize?client_id=12345&response_type=token"
    );
    let s = parse_redirect(
        "https://anilist.co/api/v2/oauth/pin#access_token=abc.def&token_type=Bearer&expires_in=31536000",
    )
    .unwrap();
    assert_eq!(s.access_token, "abc.def");
    assert_eq!(s.expires_in, Some(31_536_000));
    assert_eq!(
        parse_redirect("https://anilist.co/?error=access_denied"),
        Err(AnimeError::MissingToken)
    );
}

#[test]
fn viewer_is_parsed() {
    let v = parse_viewer(
        r#"{"data":{"Viewer":{"id":42,"name":"Senpai","avatar":{"medium":"https://a/42.png"}}}}"#,
    )
    .unwrap();
    assert_eq!(v.id, 42);
    assert_eq!(v.name, "Senpai");
    assert_eq!(v.avatar.as_deref(), Some("https://a/42.png"));
}

#[test]
fn watchlist_sorted_by_unwatched_episodes() {
    let list = parse_watching(WATCHING, TitleLanguage::Romaji).unwrap();
    let ids: Vec<u64> = list.iter().map(|e| e.media_id).collect();
    assert_eq!(ids, [1, 3, 2]);
    assert_eq!(list[0].unwatched(), 2); // Folge 6 kommt, 5 gesendet, 3 gesehen
    assert_eq!(list[1].unwatched(), 1);
    assert_eq!(list[2].unwatched(), 0);
    assert_eq!(list[0].next_episode.as_ref().unwrap().episode, 6);
}

#[test]
fn title_language_with_fallback() {
    let en = parse_watching(WATCHING, TitleLanguage::English).unwrap();
    let frieren = en.iter().find(|e| e.media_id == 1).unwrap();
    assert_eq!(frieren.title, "Frieren: Beyond Journey's End");
    let kusuriya = en.iter().find(|e| e.media_id == 2).unwrap();
    assert_eq!(
        kusuriya.title, "Kusuriya no Hitorigoto",
        "kein englischer Titel → Romaji"
    );

    let jp = parse_watching(WATCHING, TitleLanguage::Native).unwrap();
    assert_eq!(
        jp.iter().find(|e| e.media_id == 1).unwrap().title,
        "葬送のフリーレン"
    );
}

#[test]
fn save_progress_marks_completed_on_last_episode() {
    let list = parse_watching(WATCHING, TitleLanguage::Romaji).unwrap();
    let frieren = list.iter().find(|e| e.media_id == 1).unwrap();

    let req = save_progress_request(frieren, 4);
    assert_eq!(req.variables["mediaId"], 1);
    assert_eq!(req.variables["progress"], 4);
    assert_eq!(req.variables["status"], "CURRENT");

    assert_eq!(
        save_progress_request(frieren, 12).variables["status"],
        "COMPLETED"
    );

    let one_piece = list.iter().find(|e| e.media_id == 3).unwrap();
    assert_eq!(
        save_progress_request(one_piece, 9999).variables["status"],
        "CURRENT"
    );
}

#[test]
fn requests_serialize_for_http_body() {
    let body = serde_json::to_value(watching_request(42)).unwrap();
    assert_eq!(body["variables"]["userId"], 42);
    assert!(body["query"]
        .as_str()
        .unwrap()
        .contains("MediaListCollection"));
    assert!(serde_json::to_string(&viewer_request())
        .unwrap()
        .contains("Viewer"));
}

#[test]
fn api_errors_are_reported() {
    let err = parse_viewer(r#"{"data":null,"errors":[{"message":"Invalid token","status":400}]}"#)
        .unwrap_err();
    assert_eq!(err, AnimeError::Api("Invalid token".into()));
    assert!(matches!(
        parse_watching("{}", TitleLanguage::Romaji),
        Err(AnimeError::Format(_))
    ));
}
