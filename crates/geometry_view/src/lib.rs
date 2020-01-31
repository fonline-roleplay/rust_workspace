pub mod fo {
    use primitives::Hex;

    // Sprite layers
    pub const DRAW_ORDER_FLAT: u32 = 0;
    pub const DRAW_ORDER_FLAT_LAST: u32 = DRAW_ORDER - 1;
    pub const DRAW_ORDER: u32 = 20;
    pub const DRAW_ORDER_LAST: u32 = 39;
    #[repr(u32)]
    #[allow(bad_style)]
    pub enum DrawOrderType {
        DRAW_ORDER_TILE = DRAW_ORDER_FLAT + 0,
        DRAW_ORDER_TILE_END = DRAW_ORDER_FLAT + 4,
        DRAW_ORDER_HEX_GRID = DRAW_ORDER_FLAT + 5,
        DRAW_ORDER_FLAT_SCENERY = DRAW_ORDER_FLAT + 8,
        DRAW_ORDER_LIGHT = DRAW_ORDER_FLAT + 9,
        DRAW_ORDER_DEAD_CRITTER = DRAW_ORDER_FLAT + 10,
        DRAW_ORDER_FLAT_ITEM = DRAW_ORDER_FLAT + 13,
        DRAW_ORDER_TRACK = DRAW_ORDER_FLAT + 16,
        DRAW_ORDER_SCENERY = DRAW_ORDER + 3,
        DRAW_ORDER_ITEM = DRAW_ORDER + 6,
        DRAW_ORDER_CRITTER = DRAW_ORDER + 9,
        DRAW_ORDER_RAIN = DRAW_ORDER + 12,
    }

    const MAXHEX_DEF: u32 = 200;
    const MAXHEX_MIN: u32 = 10;
    const MAXHEX_MAX: u32 = 10000;
    const MAX_DRAW_ORDER: u32 = 4000200020;

    //#define DRAW_ORDER_ITEM_AUTO( i )     ( i->IsFlat() ? ( i->IsItem() ? DRAW_ORDER_FLAT_ITEM : DRAW_ORDER_FLAT_SCENERY ) : ( i->IsItem() ? DRAW_ORDER_ITEM : DRAW_ORDER_SCENERY ) )
    //#define DRAW_ORDER_CRIT_AUTO( c )     ( c->IsDead() && !c->IsRawParam( MODE_NO_FLATTEN ) ? DRAW_ORDER_DEAD_CRITTER : DRAW_ORDER_CRITTER )

    pub fn draw_order_pos_int(draw_order_type: u32, hex: Hex) -> Option<i32> {
        draw_order_pos(draw_order_type, hex)
            .map(|order| order.wrapping_sub(MAX_DRAW_ORDER / 2) as i32)
    }

    pub fn draw_order_pos(draw_order_type: u32, hex: Hex) -> Option<u32> {
        let hx = hex.x as u32;
        let hy = hex.y as u32;
        if hx > MAXHEX_MAX || hy > MAXHEX_MAX {
            return None;
        }
        match draw_order_type {
            DRAW_ORDER_FLAT..=DRAW_ORDER_FLAT_LAST => Some(
                hy * MAXHEX_MAX
                    + hx
                    + MAXHEX_MAX * MAXHEX_MAX * (draw_order_type - DRAW_ORDER_FLAT),
            ),
            DRAW_ORDER..=DRAW_ORDER_LAST => Some(
                MAXHEX_MAX * MAXHEX_MAX * DRAW_ORDER
                    + hy * DRAW_ORDER * MAXHEX_MAX
                    + hx * DRAW_ORDER
                    + (draw_order_type - DRAW_ORDER),
            ),
            _ => None,
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn test_draw_order_pos() {
            let draw_order_pos = draw_order_pos(
                DRAW_ORDER_LAST,
                Hex {
                    x: MAXHEX_MAX as u16,
                    y: MAXHEX_MAX as u16,
                },
            )
            .unwrap();
            assert_eq!(draw_order_pos, MAX_DRAW_ORDER - 1);
        }
        #[test]
        fn test_draw_order_pos_int() {
            let min = draw_order_pos_int(DRAW_ORDER_FLAT, Hex { x: 0, y: 0 }).unwrap();
            let max = draw_order_pos_int(
                DRAW_ORDER_LAST,
                Hex {
                    x: MAXHEX_MAX as u16,
                    y: MAXHEX_MAX as u16,
                },
            )
            .unwrap();

            assert!(min < max);
            assert_eq!(min, -1 * (MAX_DRAW_ORDER / 2) as i32);
            assert_eq!(max, (MAX_DRAW_ORDER / 2 - 1) as i32);
        }
    }
}
