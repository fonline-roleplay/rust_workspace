use super::AppState;
use crate::{config::Host, templates};
use actix_web::{web, HttpRequest, HttpResponse};
use clients_db::CritterInfo;
use fo_defines::CritterParam;
use fo_defines_fo4rp::param::Param;
use futures::FutureExt;
use serde::Serialize;

// TODO: Rewrite
pub async fn gm_stats(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> actix_web::Result<HttpResponse> {
    let name = req.match_info().get("client").and_then(|client| {
        percent_encoding::percent_decode(client.as_bytes())
            .decode_utf8()
            .ok()
    });
    if let Some(name) = name {
        println!("gm_stats: {:?}", name);
        let name = name.to_string();
        web::block(move || data.critters_db.client_info(&name).map(|cr| (cr, data)))
            .map(|res| {
                match res {
                    //Ok(Some(cr_info)) => Ok(format!("Your info: {:?}", cr_info).into()),
                    Ok(Ok((cr_info, data))) => match Stats::new(&cr_info).render(&data.config.host)
                    {
                        Ok(body) => Ok(HttpResponse::Ok().content_type("text/html").body(body)),
                        Err(err) => {
                            eprintln!("GM Stats error: {:#?}", err);
                            Ok(HttpResponse::InternalServerError().into())
                        }
                    },
                    _ => Ok(HttpResponse::InternalServerError().into()),
                }
            })
            .await
    } else {
        Ok(HttpResponse::Ok().body("Get out!"))
    }
}

#[derive(Debug, Serialize)]
struct Stats<'a> {
    nickname: &'a str,
    age: i32,
    sex: &'static str,
    level: i32,
    exp: i32,
    levelup_exp: i32,
    stat_fields: Vec<StatField>,
    skill_fields: Vec<SkillField>,
}

impl<'a> Stats<'a> {
    fn render(&self, host: &Host) -> Result<String, templates::TemplatesError> {
        templates::render(
            "charsheet.html",
            self,
            templates::RenderConfig { host: Some(host) },
        )
    }
}

#[derive(Debug, Serialize)]
struct StatField {
    name: &'static str,
    value: (u8, u8),
    title: &'static str,
}

#[derive(Debug, Serialize)]
struct SkillField {
    name: &'static str,
    value: i32,
    tagged: bool,
}

const STAT_TITLES: [&'static str; 10] = [
    "Гадко",
    "Плохо",
    "Низко",
    "Неплохо",
    "Средне",
    "Хорошо",
    "Высоко",
    "Отлично",
    "Круто",
    "Герой!",
];
const STAT_NAMES: [&'static str; 7] = ["СЛ", "ВC", "ВН", "ОБ", "ИН", "ЛВ", "УД"];

const SKILL_NAMES: [&'static str; 18] = [
    "Легкое оружие",
    "Тяжелое оружие",
    "Энергооружие",
    "Рукопашная",
    "Xолодное оружие",
    "Метательное оружие",
    "Санитар",
    "Доктор",
    "Скрытность",
    "Взлом замков",
    "Воровство",
    "Ловушки",
    "Наука",
    "Ремонт",
    "Красноречие",
    "Торговля",
    "Азартные игры",
    "Скиталец",
];

impl<'a> Stats<'a> {
    fn new(cr: &'a CritterInfo) -> Self {
        assert_eq!(Param::ST_MAX_LIFE as i32 - Param::ST_STRENGTH as i32, 7);

        let slice = cr.params_range_inc(Param::ST_STRENGTH..=Param::ST_LUCK);
        let stat_fields = slice
            .iter()
            .enumerate()
            .map(|(index, &st)| {
                let stat = st.max(0).min(99) as u8;
                StatField {
                    name: STAT_NAMES[index],
                    value: (stat / 10, stat % 10),
                    title: STAT_TITLES[st.max(1).min(10) as usize - 1],
                }
            })
            .collect();

        let tagged = cr.params_range_inc(Param::TAG_SKILL1..=Param::TAG_SKILL4);
        let range = Param::SK_SMALL_GUNS..=Param::SK_OUTDOORSMAN;
        let slice = cr.params_range_inc(range);
        let skill_fields = slice
            .iter()
            .enumerate()
            .map(|(index, &sk)| SkillField {
                name: SKILL_NAMES[index],
                value: sk.max(0).min(999),
                tagged: tagged
                    .iter()
                    .any(|tag| *tag as i32 == Param::SK_SMALL_GUNS as i32 + index as i32),
            })
            .collect();
        let level = cr.param(Param::ST_LEVEL);
        let next_level = level + 1;
        Stats {
            nickname: &cr.name,
            age: cr.param(Param::ST_AGE),
            sex: if cr.param(Param::ST_GENDER) == 0 {
                "МУЖ."
            } else {
                "ЖЕН."
            },
            level,
            exp: cr.param(Param::ST_EXPERIENCE),
            levelup_exp: (next_level * level / 2) * 1000,
            stat_fields,
            skill_fields,
        }
    }
}
/*
pub fn stats(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let crid = req
        .match_info()
        .get("crid")
        .and_then(|crid| crid.parse().ok());
    if let Some(crid) = crid {
        Either::A(
            data.get_ref()
                .critters_db
                .send(GetCritterInfo { id: crid })
                .from_err()
                .and_then(|res| {
                    match res {
                        //Ok(Some(cr_info)) => Ok(format!("Your info: {:?}", cr_info).into()),
                        Ok(Some(cr_info)) => {
                            if let Ok(body) = Stats::new(&cr_info).render() {
                                Ok(HttpResponse::Ok().content_type("text/html").body(body))
                            } else {
                                Ok(HttpResponse::InternalServerError().into())
                            }
                        }
                        Ok(None) => Ok("I don't know about you!".into()),
                        Err(_) => Ok(HttpResponse::InternalServerError().into()),
                    }
                }),
        )
    } else {
        Either::B(fut_ok("Get out!".into()))
    }
}
*/
