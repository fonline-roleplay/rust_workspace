use pack::puzzle::{
    piece::{Position, Template},
    pieces::Bag,
    solver::{solve, Target},
};

type Pos = (i8, i8, i8);

#[derive(Clone)]
struct Size {
    width: u8,
    height: u8,
}
impl Size {
    fn volume(&self) -> u16 {
        self.width as u16 * self.height as u16
    }
    fn brick(&self) -> Vec<Position<Pos>> {
        let mut positions = Vec::with_capacity(self.volume() as usize);
        for x in 0..self.width {
            for y in 0..self.height {
                positions.push(Position::new(x as i8, y as i8, 0));
            }
        }
        positions
    }
    fn target(&self) -> Target<Pos> {
        Target::new(self.brick())
    }
    fn template(&self) -> Template<Pos> {
        Template::new(self.brick())
    }
    fn template_with_data<D>(&self, data: D) -> Template<Pos, D> {
        Template::with_data(self.brick(), data)
    }
}

struct Inventory {
    size: Size,
}

#[derive(Clone)]
struct Item {
    size: Size,
}

struct LooseBag {
    items: Vec<Item>,
    volume: u16,
    free_volume: u16,
}
impl LooseBag {
    fn new(volume: u16) -> Self {
        Self {
            volume,
            items: Vec::new(),
            free_volume: volume,
        }
    }
    fn push(&mut self, item: Item) -> Result<(), Item> {
        match self.free_volume.checked_sub(item.size.volume()) {
            Some(free_volume) => {
                self.free_volume = free_volume;
                self.items.push(item);
                Ok(())
            }
            None => Err(item),
        }
    }
    fn push_many(&mut self, count: u8, item: Item) -> Result<(), ()> {
        match self
            .free_volume
            .checked_sub(item.size.volume() * count as u16)
        {
            Some(free_volume) => {
                self.free_volume = free_volume;
                self.items
                    .extend(std::iter::repeat(item).take(count as usize));
                Ok(())
            }
            None => Err(()),
        }
    }
    fn used_volume(&self) -> u16 {
        self.volume
            .checked_sub(self.free_volume)
            .expect("Used more volume then available")
    }
    fn bag(&self) -> Bag<Pos, u8> {
        Bag::new(
            self.items
                .iter()
                .enumerate()
                .map(|(i, item)| (1, item.size.template_with_data(i as u8)))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(width: u8, height: u8) -> Item {
        let size = Size { width, height };
        Item { size }
    }
    fn set1() -> (Inventory, LooseBag) {
        let inventory = Inventory {
            size: Size {
                width: 8,
                height: 8,
            },
        };
        let mut volume = inventory.size.volume();
        let mut loose = LooseBag::new(volume);

        loose.push_many(4, item(1, 1));
        loose.push_many(3, item(1, 2));
        loose.push_many(3, item(2, 1));
        loose.push_many(3, item(2, 2));

        loose.push_many(2, item(3, 1));
        loose.push_many(2, item(1, 3));
        loose.push_many(2, item(3, 2));
        loose.push_many(2, item(2, 3));
        assert_eq!(loose.free_volume, 0);

        (inventory, loose)
    }
    #[test]
    fn it_works() {
        let (inventory, loose_bag) = set1();
        let target = inventory.size.target();
        for (i, item) in loose_bag.items.iter().enumerate() {
            let brick = item.size.brick();
            print_positions(&item.size, Some((i, &brick[..])).into_iter());
            println!();
        }
        let bag = loose_bag.bag();
        let mut solutions = 0;
        let mut final_solution = None;
        solve(&target, bag, &mut |solution| {
            //println!("{}", solution);
            if solution.pieces().len() == loose_bag.items.len() {
                final_solution = Some(solution);
                true
            } else {
                solutions += 1;
                solutions > 100
            }
        });
        dbg!(solutions);

        //println!("{:?}", final_solution);
        if let Some(final_solution) = final_solution {
            print_positions(
                &inventory.size,
                final_solution
                    .pieces()
                    .iter()
                    .map(|piece| piece.positions())
                    .enumerate(),
            );
        }
    }
}

fn print_positions<'a>(size: &Size, iter: impl Iterator<Item = (usize, &'a [Position<Pos>])>) {
    let mut grid = vec![0; size.volume() as usize];
    for (i, piece) in iter {
        for position in piece {
            let base = position.base();
            grid[base.0 as usize + base.1 as usize * size.width as usize] = i + 1;
        }
    }
    let mut iter = grid.iter();
    for y in 0..size.height {
        for x in 0..size.width {
            let i = iter.next().unwrap();
            print!("{:03} ", i);
        }
        println!();
    }
}
