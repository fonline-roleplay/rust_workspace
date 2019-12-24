mod basic_impl;
mod critter;
mod param;
mod raw_param;

#[cfg(test)]
mod test {
    use formula::prelude::{tools::PartFormula, *};

    /*#[test]
    fn guide() {
        invar!(BASE_HP, 25, "БазовыеЖизни");
        // собираем нашу формулу
        let formula = BASE_HP + stat(Strength) * int(2) + stat(Endurance) * int(4);
        println!("Диагностический вывод формулы:\n{:#?}\n", formula);
        println!("Пользовательский вывод формулы:\n   Жизни = {}\n", formula);
        assert_eq!(
            format!("{}", formula),
            "БазовыеЖизни + Сила x 2 + Выносливость x 4"
        );
        let critter = Critter { stats: [5, 10] };
        println!(
            "Существо для которого будет произведен рассчет формулы:\n{:#?}\n",
            critter
        );
        let result = formula.compute(&critter);
        println!("Результат формулы: {:?}", result);
    }
    #[test]
    fn precedence() {
        let cr = Critter { stats: [5, 10] };
        invar!(BASE_HP, 25, "БазовыеЖизни");
        let formula1 = BASE_HP + stat(Strength) * int(2) + stat(Endurance) * int(4);
        // во второй формуле, в отличии от первой, есть скобки вокруг сложения
        let formula2 = BASE_HP + stat(Strength) * (int(2) + stat(Endurance)) * int(4);
        // если первый и второй аргумент assert_eq не равны - тест провалится
        assert_eq!(
            format!("{} = {}", formula1, formula1.compute(&cr)),
            "БазовыеЖизни + Сила x 2 + Выносливость x 4 = 75"
        );
        assert_eq!(
            format!("{} = {}", formula2, formula2.compute(&cr)),
            "БазовыеЖизни + Сила x (2 + Выносливость) x 4 = 265"
        );
        let formula3 = BASE_HP + pow((int(4) + stat(Endurance)) * stat(Strength), int(2));
        let formula3 = boxed(formula3);
        assert_eq!(
            format!("{} = {}", formula3, formula3.compute(&cr)),
            "БазовыеЖизни + ((4 + Выносливость) x Сила)^2 = 4925"
        );
    }
    #[test]
    fn cuts() {
        invar!(BASE_HP, 25, "БазовыеЖизни");
        let formula1 = BASE_HP + cut("Статы", stat(Strength) * int(2) + stat(Endurance) * int(4));

        assert_eq!(
            format!("{}", formula1),
            "БазовыеЖизни + Статы\r\nСтаты: Сила x 2 + Выносливость x 4"
        );
    }

    #[test]
    fn demo() {
        invar!(BASE_HP, 25, "БазовыеЖизни");
        invar!(HP_PER_STR, 2, "ЖизниЗаСилу");
        invar!(HP_PER_END, 4, "ЖизниЗаВыносливость");

        let cr = Critter { stats: [5, 10] };

        let formula = BASE_HP
            + "ОтСилы".part(stat(Strength) * HP_PER_STR)
            + "ОтВыносливости".part(stat(Endurance) * HP_PER_END);

        let info = formula.full_info("МаксЖизни", Some(&cr)).unwrap();

        println!("{}", info);
    }*/
    /*
    #[test]
    fn demo2() {
        // Представим что это массив Param из движка
        use crate::ops::critter::RawParam;
        let mut cr = Critter::new();
        cr.stats[RawParam::ST_STRENGTH as usize] = 5;
        cr.stats[RawParam::ST_ENDURANCE as usize] = 10;
        cr.stats[RawParam::ST_STRENGTH_EXT as usize] = 2;
        cr.stats[RawParam::ST_ENDURANCE_EXT as usize] = -3;

        // Константы
        invar!(BASE_HP, 25, "БазовыеЖизни");
        invar!(HP_PER_STR, 2, "ЖизниЗаСилу");
        invar!(HP_PER_END, 4, "ЖизниЗаВыносливость");

        // Формула
        let formula = BASE_HP
            + "ОтСилы".part(Strength.calc() * HP_PER_STR)
            + "ОтВыносливости".part(Endurance.calc() * HP_PER_END);

        // Передаём формуле массив Param'ов и просив написать красивый вывод
        let info = formula.full_info("МаксЖизни", Some(&cr)).unwrap();

        // Печатаем вывод в терминал
        println!("{}", info);
    }
    */
    #[test]
    fn basic() {
        use crate::{critter::Critter, param::*, raw_param::RawParam};

        // Представим что это массив Param из движка
        let mut param = [0i32; 1000];
        param[RawParam::ST_STRENGTH as usize] = 5;
        param[RawParam::ST_ENDURANCE as usize] = 10;
        param[RawParam::ST_STRENGTH_EXT as usize] = 2;
        param[RawParam::ST_ENDURANCE_EXT as usize] = -3;

        param[RawParam::ST_MAX_LIFE as usize] = 20;

        param[RawParam::ST_CURRENT_HP as usize] = 10;

        // Выброс адреналина работает если ТО !=0
        param[RawParam::PE_ADRENALINE_RUSH as usize] = 1;
        param[RawParam::TO_BATTLE as usize] = 1;

        let cr = Critter::new(&param);

        let formula = stat::MaxLife.calc();

        // Передаём формуле массив Param'ов и просив написать красивый вывод
        let info = formula.full_info("МаксЗдоровье", Some(&cr)).unwrap();
        //let calc = dbg!(formula.compute(&cr));

        //println!("{:#?}", formula);

        // Печатаем вывод в терминал
        println!("\n\n{}", info);

        let formula = stat::Strength.calc();
        let info = formula.full_info("Сила", Some(&cr)).unwrap();
        println!("\n\n{}", info);
    }
}
