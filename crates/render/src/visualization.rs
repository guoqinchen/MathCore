//! Math visualization - 2D function plotting
//!
//! Provides rendering for mathematical functions and expressions.

use std::fmt::Debug;

/// 2D Point
#[derive(Copy, Clone, Debug, Default)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// 2D bounding box
#[derive(Copy, Clone, Debug, Default)]
pub struct Bounds2D {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Bounds2D {
    /// Create new bounds
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Width of bounds
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Height of bounds
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Center point
    pub fn center(&self) -> Point2D {
        Point2D {
            x: (self.min_x + self.max_x) / 2.0,
            y: (self.min_y + self.max_y) / 2.0,
        }
    }

    /// Convert screen coordinate to world coordinate
    pub fn screen_to_world(
        &self,
        screen_x: f64,
        screen_y: f64,
        width: f64,
        height: f64,
    ) -> Point2D {
        Point2D {
            x: self.min_x + (screen_x / width) * self.width(),
            y: self.max_y - (screen_y / height) * self.height(),
        }
    }

    /// Convert world coordinate to screen coordinate
    pub fn world_to_screen(&self, world_x: f64, world_y: f64, width: f64, height: f64) -> Point2D {
        Point2D {
            x: ((world_x - self.min_x) / self.width()) * width,
            y: ((self.max_y - world_y) / self.height()) * height,
        }
    }
}

/// Plot style
#[derive(Debug, Clone, Default)]
pub struct PlotStyle {
    /// Line color (RGBA)
    pub color: [f32; 4],
    /// Line width in pixels
    pub line_width: f32,
    /// Fill below the curve
    pub fill: bool,
    /// Fill color
    pub fill_color: [f32; 4],
    /// Point size for scatter plots
    pub point_size: f32,
    /// Number of samples for curve evaluation
    pub samples: usize,
}

impl PlotStyle {
    /// Create a new plot style with default values
    pub fn new() -> Self {
        Self {
            color: [0.0, 0.5, 1.0, 1.0],
            line_width: 2.0,
            fill: false,
            fill_color: [0.0, 0.5, 1.0, 0.2],
            point_size: 4.0,
            samples: 500,
        }
    }
}

/// Function trait for plotting
pub trait PlotFunction: Send + Sync {
    /// Evaluate function at point x
    fn eval(&self, x: f64) -> f64;

    /// Optional: Function name for legend
    fn name(&self) -> &str {
        "f(x)"
    }

    /// Optional: Domain bounds (None = use plot bounds)
    fn domain(&self) -> Option<(f64, f64)> {
        None
    }
}

/// Function wrapper for closures
pub struct FnPlot<F: Fn(f64) -> f64 + Send + Sync> {
    f: F,
    name: String,
}

impl<F: Fn(f64) -> f64 + Send + Sync> FnPlot<F> {
    pub fn new(f: F, name: impl Into<String>) -> Self {
        Self {
            f,
            name: name.into(),
        }
    }
}

impl<F: Fn(f64) -> f64 + Send + Sync> PlotFunction for FnPlot<F> {
    fn eval(&self, x: f64) -> f64 {
        (self.f)(x)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// 2D plot data
pub struct Plot2D {
    /// Function to plot
    pub function: Box<dyn PlotFunction>,
    /// Plot bounds in world coordinates
    pub bounds: Bounds2D,
    /// Visual style
    pub style: PlotStyle,
}

impl Plot2D {
    /// Create a new plot
    pub fn new(function: impl PlotFunction + 'static, bounds: Bounds2D) -> Self {
        Self {
            function: Box::new(function),
            bounds,
            style: PlotStyle::new(),
        }
    }

    /// Create with custom style
    pub fn with_style(mut self, style: PlotStyle) -> Self {
        self.style = style;
        self
    }

    /// Sample points for the plot
    pub fn sample(&self) -> Vec<Point2D> {
        let samples = self.style.samples;
        let step = self.bounds.width() / (samples - 1) as f64;

        (0..samples)
            .map(|i| {
                let x = self.bounds.min_x + (i as f64) * step;
                let y = self.function.eval(x);
                Point2D { x, y }
            })
            .collect()
    }

    /// Check if a point is within bounds (with padding)
    pub fn is_visible(&self, point: &Point2D, padding: f64) -> bool {
        point.x >= self.bounds.min_x - padding
            && point.x <= self.bounds.max_x + padding
            && point.y >= self.bounds.min_y - padding
            && point.y <= self.bounds.max_y + padding
    }
}

/// Plot grid configuration
#[derive(Debug, Clone, Default)]
pub struct GridConfig {
    /// Number of major X divisions
    pub x_divisions: u32,
    /// Number of major Y divisions
    pub y_divisions: u32,
    /// Show minor grid lines
    pub show_minor: bool,
    /// Minor divisions per major
    pub minor_divisions: u32,
    /// Grid line color
    pub color: [f32; 4],
    /// Minor grid line color
    pub minor_color: [f32; 4],
    /// Line width
    pub line_width: f32,
}

/// Axis configuration
#[derive(Debug, Clone, Default)]
pub struct AxisConfig {
    /// Show X axis
    pub show_x: bool,
    /// Show Y axis
    pub show_y: bool,
    /// X axis position (y value)
    pub x_axis_y: f64,
    /// Y axis position (x value)
    pub y_axis_x: f64,
    /// Axis color
    pub color: [f32; 4],
    /// Line width
    pub line_width: f32,
    /// Show tick marks
    pub show_ticks: bool,
    /// Show tick labels
    pub show_labels: bool,
}

/// Plot configuration combining all options
#[derive(Debug, Clone)]
pub struct PlotConfig {
    /// Plot bounds
    pub bounds: Bounds2D,
    /// Plot style
    pub style: PlotStyle,
    /// Grid configuration
    pub grid: GridConfig,
    /// Axis configuration
    pub axis: AxisConfig,
    /// Plot title
    pub title: Option<String>,
    /// X axis label
    pub x_label: Option<String>,
    /// Y axis label
    pub y_label: Option<String>,
}

impl PlotConfig {
    /// Create new config with bounds
    pub fn new(bounds: Bounds2D) -> Self {
        Self {
            bounds,
            style: PlotStyle::new(),
            grid: GridConfig::default(),
            axis: AxisConfig::default(),
            title: None,
            x_label: Some("x".to_string()),
            y_label: Some("y".to_string()),
        }
    }

    /// Create default config for function range
    pub fn for_function(f: &dyn PlotFunction, x_min: f64, x_max: f64, samples: usize) -> Self {
        let step = (x_max - x_min) / (samples - 1) as f64;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        for i in 0..samples {
            let x = x_min + (i as f64) * step;
            let y = f.eval(x);
            if y.is_finite() {
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        }

        let y_pad = (y_max - y_min) * 0.1;
        let bounds = Bounds2D::new(x_min, y_min - y_pad, x_max, y_max + y_pad);

        Self::new(bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds() {
        let bounds = Bounds2D::new(-5.0, -10.0, 5.0, 10.0);
        assert_eq!(bounds.width(), 10.0);
        assert_eq!(bounds.height(), 20.0);
    }

    #[test]
    fn test_fn_plot() {
        let f = FnPlot::new(|x| x * x, "x^2");
        assert_eq!(f.eval(2.0), 4.0);
        assert_eq!(f.name(), "x^2");
    }
}
