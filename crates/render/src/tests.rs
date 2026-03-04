//! Integration tests for render crate

#[cfg(test)]
mod tests {
    #[test]
    fn test_render_module_imports() {
        // Test that all modules exist and can be referenced
        let _ = crate::engine::default_backend();
        let _ = crate::pipeline::Vertex2D {
            position: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let _ = crate::visualization::Bounds2D::new(0.0, 0.0, 1.0, 1.0);
        let _ = crate::window::WindowBuilder::new();
    }

    #[test]
    fn test_bounds_operations() {
        use crate::visualization::Bounds2D;

        let bounds = Bounds2D::new(-10.0, -5.0, 10.0, 5.0);
        assert_eq!(bounds.width(), 20.0);
        assert_eq!(bounds.height(), 10.0);

        let center = bounds.center();
        assert_eq!(center.x, 0.0);
        assert_eq!(center.y, 0.0);
    }

    #[test]
    fn test_bounds_coordinate_conversion() {
        use crate::visualization::Bounds2D;

        let bounds = Bounds2D::new(0.0, 0.0, 100.0, 100.0);
        let world = bounds.screen_to_world(0.0, 0.0, 100.0, 100.0);
        assert!((world.x - 0.0).abs() < 0.001);
        assert!((world.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_plot_function_trait() {
        use crate::visualization::{FnPlot, PlotFunction};

        let f = FnPlot::new(|x| x * x, "x²");
        assert_eq!(f.eval(0.0), 0.0);
        assert_eq!(f.eval(2.0), 4.0);
        assert_eq!(f.eval(-3.0), 9.0);
        assert_eq!(f.name(), "x²");
    }

    #[test]
    fn test_plot_2d_sampling() {
        use crate::visualization::{Bounds2D, FnPlot, Plot2D, PlotStyle};

        let f = FnPlot::new(|x| x, "y=x".to_string());
        let bounds = Bounds2D::new(-5.0, -5.0, 5.0, 5.0);

        let mut style = PlotStyle::new();
        style.samples = 11;

        let plot = Plot2D::new(f, bounds).with_style(style);
        let points = plot.sample();

        assert_eq!(points.len(), 11);
    }

    #[test]
    fn test_plot_visibility() {
        use crate::visualization::{Bounds2D, FnPlot, Plot2D};

        let f = FnPlot::new(|x| x * x, "x²".to_string());
        let bounds = Bounds2D::new(-5.0, 0.0, 5.0, 25.0);
        let plot = Plot2D::new(f, bounds);

        assert!(plot.is_visible(&crate::visualization::Point2D { x: 0.0, y: 0.0 }, 0.0));
        assert!(!plot.is_visible(&crate::visualization::Point2D { x: 10.0, y: 0.0 }, 0.0));
    }

    #[test]
    fn test_window_creation() {
        use crate::window::Window;

        let window = Window::new("Test", 800, 600);
        assert_eq!(window.width(), 800);
        assert_eq!(window.height(), 600);
        assert_eq!(window.title(), "Test");
    }

    #[test]
    fn test_window_resize() {
        use crate::window::Window;

        let mut window = Window::new("Test", 800, 600);
        window.resize(1024, 768);
        assert_eq!(window.width(), 1024);
        assert_eq!(window.height(), 768);
    }

    #[test]
    fn test_window_builder() {
        use crate::window::WindowBuilder;

        let window = WindowBuilder::new()
            .with_title("MathCore Viz")
            .with_size(1280, 720)
            .resizable(true)
            .decorated(true)
            .build();

        assert_eq!(window.width(), 1280);
        assert_eq!(window.height(), 720);
        assert_eq!(window.title(), "MathCore Viz");
    }

    #[test]
    fn test_error_types() {
        use crate::Error;

        let err: Error = Error::Wgpu("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err: Error = Error::Render("render error".to_string());
        assert!(err.to_string().contains("render error"));
    }

    #[test]
    fn test_vertex_layout() {
        use crate::pipeline::Vertex2D;

        let _vertex = Vertex2D {
            position: [1.0, 2.0],
            color: [0.1, 0.2, 0.3, 0.4],
        };
        assert_eq!(std::mem::size_of::<Vertex2D>(), 24);
    }

    #[test]
    fn test_uniforms_layout() {
        use crate::pipeline::Uniforms2D;

        let _uniforms = Uniforms2D::default();
        // 4x4 matrix (64 bytes) + 2 floats (8 bytes) = 72 bytes
        assert_eq!(std::mem::size_of::<Uniforms2D>(), 72);
    }

    #[test]
    fn test_grid_config_default() {
        use crate::visualization::GridConfig;

        let grid = GridConfig::default();
        // Default values are 0 (derived Default)
        assert_eq!(grid.x_divisions, 0);
        assert_eq!(grid.y_divisions, 0);
        assert!(!grid.show_minor);
    }

    #[test]
    fn test_axis_config_default() {
        use crate::visualization::AxisConfig;

        let axis = AxisConfig::default();
        // Default values are false (derived Default)
        assert!(!axis.show_x);
        assert!(!axis.show_y);
        assert_eq!(axis.x_axis_y, 0.0);
        assert_eq!(axis.y_axis_x, 0.0);
    }

    #[test]
    fn test_plot_config_creation() {
        use crate::visualization::{Bounds2D, PlotConfig};

        let bounds = Bounds2D::new(-10.0, -10.0, 10.0, 10.0);
        let config = PlotConfig::new(bounds);

        assert_eq!(config.bounds.min_x, -10.0);
        assert_eq!(config.bounds.max_x, 10.0);
    }

    #[test]
    fn test_modifiers() {
        use crate::window::Modifiers;

        let mut mods = Modifiers::default();
        assert!(!mods.shift);
        assert!(!mods.ctrl);

        mods.shift = true;
        mods.ctrl = true;
        assert!(mods.shift);
        assert!(mods.ctrl);
    }

    #[test]
    fn test_key_variants() {
        use crate::window::Key;

        assert_ne!(Key::A, Key::B);
        assert_eq!(Key::ArrowUp, Key::ArrowUp);
    }

    #[test]
    fn test_mouse_button_variants() {
        use crate::window::MouseButton;

        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_ne!(MouseButton::Left, MouseButton::Right);
    }

    #[tokio::test]
    async fn test_engine_config_default() {
        use crate::engine::EngineConfig;

        let config = EngineConfig::default();
        assert!(config.vsync);
        assert!(config.high_dpi);
    }

    #[test]
    fn test_pipeline_config_default() {
        use crate::pipeline::PipelineConfig;

        let config = PipelineConfig::default();
        assert_eq!(config.sample_count, 1);
    }
}
