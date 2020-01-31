mod ops;
pub mod prelude;

// тестовый модуль, собирается и выполняется только по команде cargo test
#[cfg(test)]
mod test {
    use crate::ops::{Descriptor, PartFormula};
    use crate::prelude::{tools::*, *};

    #[test]
    #[allow(unused_variables)]
    fn data_conversion() {
        #[derive(Copy, Clone)]
        struct Foo;
        impl FormulaData for &Foo {};
        invar!(INVAR, 50, "invar");

        {
            let formula = INVAR;
            assert_eq!(formula.compute(()), 50);
            assert_eq!(formula.compute(&Foo), 50);
        }
        {
            #[derive(Debug, Clone)]
            struct Bar;
            impl Formula<&Foo, i32> for Bar {
                fn compute(&self, input: &Foo) -> i32 {
                    100
                }
                fn description<D: Descriptor>(
                    &self,
                    desc: &mut D,
                    input: Option<&Foo>,
                ) -> std::fmt::Result {
                    unimplemented!()
                }
            }
            let bar: Op<&Foo, i32, Bar> = op(Bar);
            assert_eq!(bar.compute(&Foo), 100);

            {
                assert_eq!(
                    {
                        let formula: Op<&Foo, i32, _> = bar.clone() + bar.clone();
                        formula.compute(&Foo)
                    },
                    200
                );
                assert_eq!(
                    {
                        let formula: Op<&Foo, i32, _> = (bar.clone() + bar.clone()).compat();
                        formula.compute(&Foo)
                    },
                    200
                );
                assert_eq!(
                    {
                        let formula: Op<&Foo, i32, _> = (INVAR + INVAR).compat();
                        formula.compute(&Foo)
                    },
                    100
                );
            }

            {
                assert_eq!(
                    {
                        let formula: Op<&Foo, i32, _> = INVAR + bar.clone();
                        formula.compute(&Foo)
                    },
                    150
                );
                assert_eq!(
                    {
                        let formula: Op<&Foo, i32, _> = bar.clone() + INVAR;
                        formula.compute(&Foo)
                    },
                    150
                );
            }
            {
                assert_eq!(
                    {
                        let formula: Op<&Foo, i32, _> = int(1) + bar.clone();
                        formula.compute(&Foo)
                    },
                    101
                );
                assert_eq!(
                    {
                        let formula: Op<&Foo, i32, _> = bar.clone() + int(1);
                        formula.compute(&Foo)
                    },
                    101
                );
            }
        }
    }

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
}
