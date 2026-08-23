use crate::config::Area;
use crate::simulation::Particle;

/// Cells are never narrower than the widest possible contact distance, which
/// is what guarantees an overlapping pair lands in the 3x3 block around either.
pub struct SpatialGrid {
    origin: [f32; 2],
    cell_size: f32,
    columns: usize,
    rows: usize,
    cells: Vec<Vec<u32>>,
}

impl SpatialGrid {
    pub fn new(particles: &[Particle], area: Area, widest_radius: f32, padding: f32) -> Self {
        let (semi_x, semi_y) = area.semi_axes();
        let width = (2.0 * semi_x).max(1e-3);
        let height = (2.0 * semi_y).max(1e-3);

        let contact = 2.0 * widest_radius + padding;
        let even_spread = (width * height / particles.len() as f32).sqrt();
        let cell_size = contact.max(even_spread).max(1e-3);

        let columns = ((width / cell_size).ceil() as usize + 1).max(1);
        let rows = ((height / cell_size).ceil() as usize + 1).max(1);

        Self {
            origin: [-semi_x, -semi_y],
            cell_size,
            columns,
            rows,
            cells: vec![Vec::new(); columns * rows],
        }
    }

    pub fn rebuild(&mut self, particles: &[Particle]) {
        for cell in &mut self.cells {
            cell.clear();
        }

        for (i, particle) in particles.iter().enumerate() {
            let (column, row) = self.cell_of(*particle.position());
            self.cells[row * self.columns + column].push(i as u32);
        }
    }

    pub fn neighbours(&self, position: [f32; 2]) -> impl Iterator<Item = usize> {
        let (column, row) = self.cell_of(position);

        (-1..=1isize)
            .flat_map(|dy| (-1..=1isize).map(move |dx| (dx, dy)))
            .filter_map(move |(dx, dy)| {
                let x = column as isize + dx;
                let y = row as isize + dy;
                let inside =
                    x >= 0 && y >= 0 && (x as usize) < self.columns && (y as usize) < self.rows;
                inside.then(|| y as usize * self.columns + x as usize)
            })
            .flat_map(move |cell| self.cells[cell].iter().map(|&i| i as usize))
    }

    fn cell_of(&self, position: [f32; 2]) -> (usize, usize) {
        let x = ((position[0] - self.origin[0]) / self.cell_size).floor();
        let y = ((position[1] - self.origin[1]) / self.cell_size).floor();

        (
            (x.max(0.0) as usize).min(self.columns - 1),
            (y.max(0.0) as usize).min(self.rows - 1),
        )
    }
}
