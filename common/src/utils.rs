pub mod map {
    use crate::primitives::*;
    const SQRT3T2_FLOAT: f32 = 3.4641016151;
    const SQRT3_FLOAT: f32 = 1.732050807568877;
    const RAD2DEG: f32 = 57.29577951;

    fn get_distance(x1: i32, y1: i32, x2: i32, y2: i32, hexagonal: bool) -> u32 {
        if hexagonal {
            let dx = if x1 > x2 { x1 - x2 } else { x2 - x1 };
            let rx = if x1 & 1 == 0 {
                if y2 <= y1 {
                    y1 - y2 - dx / 2
                } else {
                    y2 - y1 - (dx + 1) / 2
                }
            } else {
                if y2 >= y1 {
                    y2 - y1 - dx / 2
                } else {
                    y1 - y2 - (dx + 1) / 2
                }
            };
            (if rx > 0 { dx + rx } else { dx }) as u32
        } else {
            let dx = i32::abs(x2 - x1) as u32;
            let dy = i32::abs(y2 - y1) as u32;
            u32::max(dx, dy)
        }
    }
    pub fn get_distance_hex(begin_hex: Hex, end_hex: Hex, hexagonal: bool) -> u32 {
        get_distance(
            begin_hex.x as i32,
            begin_hex.y as i32,
            end_hex.x as i32,
            end_hex.y as i32,
            hexagonal,
        )
    }

    fn get_far_dir(x1: i32, y1: i32, x2: i32, y2: i32, hexagonal: bool) -> u8 {
        if hexagonal {
            let hx = x1 as f32;
            let hy = y1 as f32;
            let tx = x2 as f32;
            let ty = y2 as f32;
            let nx = 3.0 * (tx - hx);
            let ny = (ty - hy) * SQRT3T2_FLOAT - ((x2 & 1) as f32 - (x1 & 1) as f32) * SQRT3_FLOAT;
            let dir = 180.0 + RAD2DEG * f32::atan2(ny, nx);

            if dir >= 60.0 && dir < 120.0 {
                5
            } else if dir >= 120.0 && dir < 180.0 {
                4
            } else if dir >= 180.0 && dir < 240.0 {
                3
            } else if dir >= 240.0 && dir < 300.0 {
                2
            } else if dir >= 300.0 {
                1
            } else {
                0
            }
        } else {
            /*float dir = 180.0f + RAD2DEG* atan2( (float) ( x2 - x1 ), (float) ( y2 - y1 ) );

            if( dir >= 22.5f  && dir < 67.5f )
            return 7;
            if( dir >= 67.5f  && dir < 112.5f )
            return 0;
            if( dir >= 112.5f && dir < 157.5f )
            return 1;
            if( dir >= 157.5f && dir < 202.5f )
            return 2;
            if( dir >= 202.5f && dir < 247.5f )
            return 3;
            if( dir >= 247.5f && dir < 292.5f )
            return 4;
            if( dir >= 292.5f && dir < 337.5f )
            return 5;
            return 6;*/
            unimplemented!()
        }
    }

    pub fn get_direction(from_hex: Hex, to_hex: Hex, hexagonal: bool) -> u8 {
        get_far_dir(
            from_hex.x as i32,
            from_hex.y as i32,
            to_hex.x as i32,
            to_hex.y as i32,
            hexagonal,
        )
    }

    fn move_hex_by_dir(hex: Hex, dir: u8, max: Hex) -> Option<Hex> {
        let (hxi, hyi) = MoveHexByDirUnsafe((hex.x as i32, hex.y as i32), dir);

        if hxi >= 0 && hxi < max.x as i32 && hyi >= 0 && hyi < max.y as i32 {
            Some(Hex {
                x: hxi as u16,
                y: hyi as u16,
            })
        } else {
            None
        }
    }

    fn MoveHexByDirUnsafe((mut hx, mut hy): (i32, i32), dir: u8) -> (i32, i32) {
        if (true) {
            //GameOpt.MapHexagonal )
            match dir {
                0 => {
                    hx -= 1;
                    if hx & 1 == 0 {
                        hy -= 1;
                    }
                }
                1 => {
                    hx -= 1;
                    if hx & 1 != 0 {
                        hy += 1;
                    }
                }
                2 => {
                    hy += 1;
                }
                3 => {
                    hx += 1;
                    if hx & 1 != 0 {
                        hy += 1;
                    }
                }
                4 => {
                    hx += 1;
                    if hx & 1 == 0 {
                        hy -= 1;
                    }
                }
                5 => {
                    hy -= 1;
                }
                _ => panic!("Invalid direction"),
            }
        } else {
            unimplemented!()
            /*switch( dir )
            {
                case 0:
                hx--;
                break;
                case 1:
                hx--;
                hy++;
                break;
                case 2:
                hy++;
                break;
                case 3:
                hx++;
                hy++;
                break;
                case 4:
                hx++;
                break;
                case 5:
                hx++;
                hy--;
                break;
                case 6:
                hy--;
                break;
                case 7:
                hx--;
                hy--;
                break;
                default:
                return;
            }*/
        }
        (hx, hy)
    }

    struct LineTracer {
        max: Hex,
        x1: f32,
        y1: f32,
        //x2: f32,
        //y2: f32,
        //dir: f32,
        dir1: u8,
        dir2: u8,
        dx: f32,
        dy: f32,
    }
    impl LineTracer {
        pub fn new(from: Hex, to: Hex, max: Hex, angle: f32, is_square: bool) -> Self {
            /*Self{
                max,
            }*/
            if is_square {
                unimplemented!()
            /*dir = atan2( (float) ( ty - hy ), (float) ( tx - hx ) ) + angle;
            dx = cos( dir );
            dy = sin( dir );
            if( fabs( dx ) > fabs( dy ) )
            {
                dy /= fabs( dx );
                dx = ( dx > 0 ? 1.0f : -1.0f );
            }
            else
            {
                dx /= fabs( dy );
                dy = ( dy > 0 ? 1.0f : -1.0f );
            }
            x1 = (float) hx + 0.5f;
            y1 = (float) hy + 0.5f;*/
            } else {
                const BIAS_FLOAT: f32 = 0.02;

                let nx = 3.0 * (to.x as f32 - from.x as f32);
                let ny = (to.y as f32 - from.y as f32) * SQRT3T2_FLOAT
                    - ((to.x & 1) as f32 - (from.x & 1) as f32) * SQRT3_FLOAT;
                let mut dir = 180.0 + RAD2DEG * f32::atan2(ny, nx);
                if angle != 0.0 {
                    dir = Self::normalize_dir(dir + angle);
                }

                let (dir1, dir2) = if dir >= 30.0 && dir < 90.0 {
                    (5, 0)
                } else if dir >= 90.0 && dir < 150.0 {
                    (4, 5)
                } else if dir >= 150.0 && dir < 210.0 {
                    (3, 4)
                } else if dir >= 210.0 && dir < 270.0 {
                    (2, 3)
                } else if dir >= 270.0 && dir < 330.0 {
                    (1, 2)
                } else {
                    (0, 1)
                };

                let x1 = 3.0 * from.x as f32 + BIAS_FLOAT;
                let y1 =
                    SQRT3T2_FLOAT * from.y as f32 - SQRT3_FLOAT * (from.x & 1) as f32 + BIAS_FLOAT;
                let mut x2 = 3.0 * to.x as f32 + BIAS_FLOAT + BIAS_FLOAT;
                let mut y2 =
                    SQRT3T2_FLOAT * to.y as f32 - SQRT3_FLOAT * (to.x & 1) as f32 + BIAS_FLOAT;

                if angle != 0.0 {
                    x2 -= x1;
                    y2 -= y1;
                    let xp = f32::cos(angle / RAD2DEG) * x2 - f32::sin(angle / RAD2DEG) * y2;
                    let yp = f32::sin(angle / RAD2DEG) * x2 + f32::cos(angle / RAD2DEG) * y2;
                    x2 = x1 + xp;
                    y2 = y1 + yp;
                }
                let dx = x2 - x1;
                let dy = y2 - y1;

                LineTracer {
                    max,
                    x1,
                    y1,
                    dir1,
                    dir2,
                    dx,
                    dy,
                }
            }
        }
        fn dist(&self, hex: Hex) -> f32 {
            f32::abs(
                self.dx
                    * (self.y1 - (SQRT3T2_FLOAT * hex.y as f32 - (hex.x & 1) as f32 * SQRT3_FLOAT))
                    - self.dy * (self.x1 - 3.0 * hex.x as f32),
            )
        }
        fn get_next_hex(&self, hex: &mut Hex) -> u8 {
            let t1 = move_hex_by_dir(*hex, self.dir1, self.max);
            let t2 = move_hex_by_dir(*hex, self.dir2, self.max);
            match (t1, t2) {
                (Some(t1), Some(t2)) => {
                    let dist1 = self.dist(t1);
                    let dist2 = self.dist(t2);
                    if dist1 <= dist2 {
                        // Left hand biased
                        *hex = t1;
                        self.dir1
                    } else {
                        *hex = t2;
                        self.dir2
                    }
                }
                (Some(t1), None) => {
                    *hex = t1;
                    self.dir1
                }
                (None, Some(t2)) => {
                    *hex = t2;
                    self.dir2
                }
                (None, None) => 0,
            }
        }
        //uchar GetNextHex( ushort& cx, ushort& cy );
        //void  GetNextSquare( ushort& cx, ushort& cy );
        fn normalize_dir(dir: f32) -> f32 {
            use std::ops::Rem;
            if dir <= 0.0 {
                360.0 - f32::rem(-dir, 360.0)
            } else if dir >= 0.0 {
                f32::rem(dir, 360.0)
            } else {
                dir
            }
        }
    }

    #[cfg(feature = "server")]
    pub mod server {
        use super::*;
        use crate::engine_types::{critter::Critter, map::Map};

        struct TraceInput<'a> {
            // Input
            map: &'a Map,
            begin_hex: Hex,
            end_hex: Hex,
            angle: f32,
            dist: u32,
            //find_critter: Critter *;
            find_type: i32,
            is_check_team: bool,
            base_cr_team_id: u32,
            want_last_passed: bool,
            last_passed_skip_critters: bool,
            hexagonal: bool,
            //bool        ( * HexCallback )( Map *, Critter*, ushort, ushort, ushort, ushort, uchar );
        }
        impl<'a> TraceInput<'a> {
            fn new(map: &'a Map, begin_hex: Hex, end_hex: Hex, angle: f32, dist: u32) -> Self {
                Self {
                    map,
                    begin_hex,
                    end_hex,
                    dist,
                    angle,
                    find_type: 0,
                    is_check_team: false,
                    base_cr_team_id: 0,
                    want_last_passed: false,
                    last_passed_skip_critters: false,
                    hexagonal: true,
                }
            }
            fn trace(&self) -> TraceOutput {
                let max_hex = self.map.get_max_hex();

                let dist = if self.dist == 0 {
                    get_distance_hex(self.begin_hex, self.end_hex, self.hexagonal)
                } else {
                    self.dist
                };

                let mut output = TraceOutput {
                    //critters: Vec::new(),
                    pre_block: self.begin_hex, //old //old_c
                    block: self.begin_hex,     //currenct //c
                    last_passed: None,
                    is_full_trace: false,
                    is_critter_founded: false,
                    is_teammate_founded: false,
                };
                let mut dir = 0u8;

                //LineTracer line_tracer( hx, hy, tx, ty, maxhx, maxhy, trace.Angle, !GameOpt.MapHexagonal );
                let line_tracer = LineTracer::new(
                    self.begin_hex,
                    self.end_hex,
                    max_hex,
                    self.angle,
                    !self.hexagonal,
                );

                let mut last_passed_ok = false;
                for i in 0.. {
                    if i >= dist {
                        output.is_full_trace = true;
                        break;
                    }

                    if self.hexagonal {
                        dir = line_tracer.get_next_hex(&mut output.block);
                    } else {
                        unimplemented!()
                        //line_tracer.get_next_square( output.block );
                        //dir = get_near_dir( output.pre_block, output.block );
                    }

                    /*if self.hex_callback {
                        trace.HexCallback( map, trace.FindCr, old_cx, old_cy, cx, cy, dir );
                        old_cx = cx;
                        old_cy = cy;
                        continue;
                    }*/

                    if self.want_last_passed && !last_passed_ok {
                        if self.map.is_hex_passed(output.block) {
                            output.last_passed = Some(output.block);
                        } else if !self.map.is_hex_critter(output.block)
                            || !self.last_passed_skip_critters
                        {
                            last_passed_ok = true;
                        }
                    }

                    if !self.map.is_hex_raked(output.block) {
                        break;
                    }

                    /*if( trace.Critters != NULL && map->IsHexCritter( cx, cy ) ) {
                        map -> GetCrittersHex(cx, cy, 0, trace.FindType, *trace.Critters, false);
                    }*/
                    /*if( ( trace.FindCr || trace.IsCheckTeam ) && map->IsFlagCritter( cx, cy, false ) ) {
                        Critter* cr = map->GetHexCritter( cx, cy, false, false );
                        if( cr ) {
                            if( cr == trace.FindCr ) {
                                trace.IsCritterFounded = true;
                                break;
                            }
                            if( trace.IsCheckTeam && cr->Data.Params[ ST_TEAM_ID ] == (int) trace.BaseCrTeamId )
                            {
                                trace.IsTeammateFounded = true;
                                break;
                            }
                        }
                    }*/
                    output.pre_block = output.block;
                }
                output
            }
        }
        #[derive(Debug)]
        struct TraceOutput {
            //critters: Vec<&'a Critter>,
            pre_block: Hex,
            block: Hex,
            last_passed: Option<Hex>,
            is_full_trace: bool,
            is_critter_founded: bool,
            is_teammate_founded: bool,
        }

        pub fn get_hex_in_path_wall(
            map: &Map,
            begin_hex: Hex,
            end_hex: Hex,
            angle: f32,
            dist: u32,
        ) -> Hex {
            let mut trace = TraceInput::new(map, begin_hex, end_hex, angle, dist);
            trace.want_last_passed = true;
            let output = trace.trace();
            if let Some(last_passed) = output.last_passed {
                last_passed
            } else {
                begin_hex
            }
        }

        pub fn get_hex_in_path(
            map: &Map,
            begin_hex: Hex,
            end_hex: Hex,
            angle: f32,
            dist: u32,
        ) -> Hex {
            let mut trace = TraceInput::new(map, begin_hex, end_hex, angle, dist);
            let output = trace.trace();
            output.pre_block
        }
    }
}
