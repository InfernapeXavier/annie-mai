use html2md::parse_html;
use serde::Deserialize;
use titlecase::titlecase;

use crate::utils::formatter::{code, linker};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct Anime {
    id: u32,
    id_mal: u32,
    title: Title,
    season: Option<String>,
    season_year: Option<u32>,
    format: Option<String>,
    status: Option<String>,
    episodes: Option<u32>,
    duration: Option<u32>,
    genres: Vec<String>,
    source: Option<String>,
    cover_image: CoverImage,
    average_score: Option<u32>,
    studios: Option<Studios>,
    site_url: String,
    external_links: Option<Vec<ExternalLinks>>,
    trailer: Option<Trailer>,
    description: String,
}

#[derive(Deserialize, Debug)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]

pub struct CoverImage {
    pub extra_large: String,
    pub large: String,
    pub medium: String,
    pub color: String,
}

#[derive(Deserialize, Debug)]
pub struct Studios {
    pub edges: Vec<Edges>,
    pub nodes: Vec<Nodes>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Edges {
    pub id: u32,
    pub is_main: bool,
}

#[derive(Deserialize, Debug)]
pub struct Nodes {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct ExternalLinks {
    pub url: String,
    #[serde(alias = "type")]
    pub url_type: String,
}

#[derive(Deserialize, Debug)]
pub struct Trailer {
    pub id: String,
    pub site: String,
}

impl Anime {
    pub fn transform_mal_id(&self) -> String {
        format!("https://www.myanimelist.com/anime/{}", self.id_mal)
    }

    pub fn transform_title(&self) -> String {
        match &self.title.romaji {
            Some(title) => title.to_string(),
            None => match &self.title.english {
                Some(title) => title.to_string(),
                None => self.title.native.as_ref().unwrap().to_string(),
            },
        }
    }

    pub fn transform_season(&self) -> String {
        let season = match &self.season {
            Some(season) => season.to_string(),
            None => "".to_string(),
        };
        let year = match &self.season_year {
            Some(year) => year.to_string(),
            None => "".to_string(),
        };

        let built_string = vec![season, year];
        let return_string = titlecase(built_string.join(" ").trim());

        match return_string {
            _ if return_string.is_empty() => "-".to_string(),
            _ => return_string,
        }
    }

    pub fn transform_format(&self) -> String {
        match &self.format {
            Some(format) => match format.to_string() {
                _ if *format == "TV" => format.to_string(),
                _ => titlecase(format),
            },
            None => "-".to_string(),
        }
    }

    pub fn transform_status(&self) -> String {
        match &self.status {
            Some(status) => match status {
                _ if status.starts_with("NOT") => titlecase("Not Released"),
                _ => titlecase(status),
            },
            None => "-".to_string(),
        }
    }

    pub fn transform_episodes(&self) -> String {
        match &self.episodes {
            Some(episodes) => episodes.to_string(),
            None => "-".to_string(),
        }
    }

    pub fn transform_duration(&self) -> String {
        match &self.duration {
            Some(duration) => format!("{} mins", duration),
            None => "-".to_string(),
        }
    }

    pub fn transform_genres(&self) -> String {
        let genres = self
            .genres
            .clone()
            .into_iter()
            .map(|genre| code(titlecase(&genre)))
            .collect::<Vec<String>>();
        genres.join(" - ")
    }

    pub fn transform_source(&self) -> String {
        match &self.source {
            Some(source) => titlecase(source),
            None => "-".to_string(),
        }
    }

    // CoverImage Transformers
    pub fn transform_color(&self) -> i32 {
        i32::from_str_radix(self.cover_image.color.trim_start_matches('#'), 16).unwrap_or(0x0000ff)
    }

    pub fn transform_thumbnail(&self) -> String {
        self.cover_image.large.to_string()
    }

    pub fn transform_score(&self) -> String {
        match &self.average_score {
            Some(score) => format!("{}/100", score),
            None => "-".to_string(),
        }
    }

    pub fn transform_studios(&self) -> String {
        if self.studios.is_none() {
            return "-".to_string();
        }

        let studios = &self.studios.as_ref().unwrap();
        let mut main_studio_indices: Vec<usize> = Vec::new();

        for (index, edge) in studios.edges.iter().enumerate() {
            if edge.is_main {
                main_studio_indices.push(index);
            }
        }

        let mut main_studios: Vec<String> = Vec::new();

        for main_studio_index in main_studio_indices {
            main_studios.push(studios.nodes[main_studio_index].name.to_string())
        }

        let main_studios = main_studios
            .clone()
            .into_iter()
            .map(|studio| code(titlecase(&studio)))
            .collect::<Vec<String>>();

        main_studios.join(" x ")
    }

    pub fn transform_anilist(&self) -> String {
        self.site_url.to_string()
    }

    pub fn transform_links(&self) -> String {
        match &self.external_links {
            Some(links) => {
                if links.is_empty() {
                    "-".to_string()
                } else {
                    links
                        .iter()
                        .filter(|link| link.url_type.to_lowercase() == "streaming")
                        .map(|link| link.url.to_string())
                        .collect::<Vec<String>>()
                        .into_iter()
                        .filter(|link| match link {
                            _ if link.contains("hbo") => true,
                            _ if link.contains("netflix") => true,
                            _ if link.contains("crunchyroll") => true,
                            _ => false,
                        })
                        .collect::<Vec<String>>()
                        .into_iter()
                        .map(|link| match link {
                            _ if link.contains("hbo") => linker("HBO".to_string(), link),
                            _ if link.contains("netflix") => linker("Netflix".to_string(), link),
                            _ if link.contains("crunchyroll") => {
                                linker("Crunchyroll".to_string(), link)
                            }
                            _ => "Invalid".to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join(" ")
                }
            }
            None => "-".to_string(),
        }
    }

    pub fn transform_trailer(&self) -> String {
        match &self.trailer {
            None => String::from("None"),
            Some(trailer) => {
                let url: String =
                    format!("https://www.{}.com/watch?v={}", trailer.site, trailer.id);
                linker("YouTube".to_string(), url)
            }
        }
    }

    pub fn transform_description(&self) -> String {
        parse_html(&self.description.to_string())
    }
}