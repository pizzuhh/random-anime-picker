use serde_json::*;
use serde::*;
use reqwest::*;
//use rustyline::*;
//use rand::*;
use rand::seq::SliceRandom;

//get user id
const QUERY1: &str = r#"
query($id:Int,$name:String){User(id:$id,name:$name){id name previousNames{name updatedAt}avatar{large}bannerImage about isFollowing isFollower donatorTier donatorBadge createdAt moderatorRoles isBlocked bans options{profileColor restrictMessagesToFollowing}mediaListOptions{scoreFormat}statistics{anime{count meanScore standardDeviation minutesWatched episodesWatched genrePreview:genres(limit:10,sort:COUNT_DESC){genre count}}manga{count meanScore standardDeviation chaptersRead volumesRead genrePreview:genres(limit:10,sort:COUNT_DESC){genre count}}}stats{activityHistory{date amount level}}favourites{anime{edges{favouriteOrder node{id type status(version:2)format isAdult bannerImage title{userPreferred}coverImage{large}startDate{year}}}}manga{edges{favouriteOrder node{id type status(version:2)format isAdult bannerImage title{userPreferred}coverImage{large}startDate{year}}}}characters{edges{favouriteOrder node{id name{userPreferred}image{large}}}}staff{edges{favouriteOrder node{id name{userPreferred}image{large}}}}studios{edges{favouriteOrder node{id name}}}}}}
"#;

//get anime list
const QUERY2: &str = r#"
query($userId:Int,$userName:String,$type:MediaType){MediaListCollection(userId:$userId,userName:$userName,type:$type){lists{name isCustomList isCompletedList:isSplitCompletedList entries{...mediaListEntry}}user{id name avatar{large}mediaListOptions{scoreFormat rowOrder animeList{sectionOrder customLists splitCompletedSectionByFormat theme}mangaList{sectionOrder customLists splitCompletedSectionByFormat theme}}}}}fragment mediaListEntry on MediaList{id mediaId status score progress progressVolumes repeat priority private hiddenFromStatusLists customLists advancedScores notes updatedAt startedAt{year month day}completedAt{year month day}media{id title{userPreferred romaji english native}coverImage{extraLarge large}type format status(version:2)episodes volumes chapters averageScore popularity isAdult countryOfOrigin genres bannerImage startDate{year month day}}}
"#;

const QUERY3:&str = r#"
   query ($id: Int) {
  Media (id: $id, type: ANIME) {
    id
    title {
      romaji
      english
      native
    }
    format
  }
}
"#;

const URL: &str = "https://graphql.anilist.co";

async fn get_anime_info(id: &i64)
{
    #[derive(Serialize, Deserialize, Debug)]
    struct Json
    {
        data: Wrap
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Wrap
    {
        #[serde(rename="Media")]
        media:Media
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Media
    {
        title: Title,
        format: String
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Title
    {
        english: Option<String>,
        native: Option<String>,
        romaji: Option<String>
    }

    let client = Client::new();
    let json = json!({"query": QUERY3, "variables": {"id": id}});
    let resp = client.post(URL)
        .header("Content-Type", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;
    let result:Json = serde_json::from_str(&resp.unwrap()).unwrap();
    let link = format!("https://anilist.co/anime/{}", id);
    //println!("anime -> {}\nromaji -> {}\nenglish -> {}\nnative -> {}", link, result.data.media.title.romaji, result.data.media.title.english, result.data.media.title.native);
    println!("anime -> {}\nromaji -> {}\nenglish -> {}\nnative -> {}\nformat -> {}", link,
        if let Some(romaji) = result.data.media.title.romaji {romaji} else {"N/A".to_string()},
        if let Some(english) = result.data.media.title.english {english} else {"N/A".to_string()},
        if let Some(native) = result.data.media.title.native {native.to_string()} else {"N/A".to_string()}, result.data.media.format);
}

async fn get_anime_ids(usrid: i32) -> Vec<i64>
{

    #[derive(Serialize, Deserialize, Debug)]
    struct Json
    {
        data: Wrapper
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Wrapper
    {
        #[serde(rename = "MediaListCollection")]
        media_list_collection: MLC
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct MLC
    {
        lists: Vec<Lists>
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Lists
    {
        name: String,
        entries: Vec<Entr>
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Entr
    {
        #[serde(rename = "mediaId")]
        id: i64,
        media: Media
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Media
    {
        format: Option<String>
    }
    let cl = Client::new();
    let qer = json!({"query": QUERY2, "variables":{"type":"ANIME","userId":usrid}});

    let resp = cl.post(URL)
        .header("Content-Type", "application/json")
        .body(qer.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let parsed: Json = serde_json::from_str(&resp.unwrap()).unwrap();
    let mut ids: Vec<i64> = Vec::new();
    /*
    hardcoding like parsed.data.media_list_collection.lists.get(2) is not good!
    */
    for i in parsed.data.media_list_collection.lists
    {
        if i.name == "Planning"
        {
            for ent in i.entries
            {
                if ent.media.format == Some("TV".to_string()) || ent.media.format == Some("MOVIE".to_string())
                {
                    ids.push(ent.id);
                }
            }
        }
    }
    return ids;
    
}

async fn get_user_id(name: String) -> i32
{
    #[derive(Serialize, Deserialize, Debug)]
    struct ID
    {
        data: Wrap
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Wrap
    {
        #[serde(rename="User")]
        user: Usr
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Usr
    {
        id: i32
    }
    let cl = Client::new();
    let qer = json!({"query":QUERY1, "variables":{"name": name}});
    let resp = cl.post(URL)
        .header("Content-Type", "application/json")
        .body(qer.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;
    let id: ID = serde_json::from_str(&resp.unwrap()).unwrap();
    return id.data.user.id;
}

#[tokio::main]
async fn main()
{
    let rl = rustyline::DefaultEditor::new();
    let username = rl.expect("error reading input").readline("anilist username: ");
    let mut id: i32 = 0;
    match username
    {
        Ok(name) => 
        {
            id = get_user_id(name).await;
        }, Err(e) => println!("{:?}", e)
    }
    let wtf = get_anime_ids(id).await;
    let rnd = wtf.choose(&mut rand::thread_rng());
    println!("anime id -> {}", rnd.unwrap());
    get_anime_info(rnd.unwrap()).await;
}
