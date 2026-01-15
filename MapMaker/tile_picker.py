import json
import math
import os
import tkinter as tk
from tkinter import filedialog, messagebox
from PIL import Image, ImageTk

# ---------------------------
# Helpers
# ---------------------------

def clamp(v, lo, hi):
    return max(lo, min(hi, v))

def compute_output_grid(n_tiles, cols_pref=None):
    """Choose output columns/rows. If cols_pref is None, choose near-square."""
    if n_tiles <= 0:
        return (0, 0)
    if cols_pref and cols_pref > 0:
        cols = cols_pref
    else:
        cols = int(math.ceil(math.sqrt(n_tiles)))
    rows = int(math.ceil(n_tiles / cols))
    return cols, rows

# ---------------------------
# App
# ---------------------------

class TilesetPickerApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Tileset Picker -> Compact Tileset Exporter")

        # State
        self.img_path = None
        self.img = None              # PIL image
        self.img_w = 0
        self.img_h = 0

        self.tile_w = tk.IntVar(value=16)
        self.tile_h = tk.IntVar(value=16)
        self.out_cols = tk.IntVar(value=0)   # 0 = auto
        self.zoom = tk.DoubleVar(value=2.0)

        self.tiles_x = 0
        self.tiles_y = 0

        self.selected = set()  # set of (tx, ty) in source grid

        # Display
        self.tk_img = None
        self.canvas_img_id = None

        # UI layout
        self._build_ui()

    def _build_ui(self):
        top = tk.Frame(self.root, padx=8, pady=8)
        top.pack(side=tk.TOP, fill=tk.X)

        btn_load = tk.Button(top, text="Load Tileset PNG", command=self.load_image)
        btn_load.grid(row=0, column=0, padx=4, pady=4, sticky="w")

        tk.Label(top, text="Tile W").grid(row=0, column=1, padx=4, pady=4)
        tk.Entry(top, textvariable=self.tile_w, width=6).grid(row=0, column=2, padx=4, pady=4)

        tk.Label(top, text="Tile H").grid(row=0, column=3, padx=4, pady=4)
        tk.Entry(top, textvariable=self.tile_h, width=6).grid(row=0, column=4, padx=4, pady=4)

        tk.Label(top, text="Zoom").grid(row=0, column=5, padx=4, pady=4)
        tk.Entry(top, textvariable=self.zoom, width=6).grid(row=0, column=6, padx=4, pady=4)

        tk.Label(top, text="Output Cols (0=auto)").grid(row=0, column=7, padx=4, pady=4)
        tk.Entry(top, textvariable=self.out_cols, width=8).grid(row=0, column=8, padx=4, pady=4)

        btn_apply = tk.Button(top, text="Apply Grid / Redraw", command=self.redraw)
        btn_apply.grid(row=0, column=9, padx=4, pady=4, sticky="w")

        btn_clear = tk.Button(top, text="Clear Selection", command=self.clear_selection)
        btn_clear.grid(row=0, column=10, padx=4, pady=4, sticky="w")

        btn_export = tk.Button(top, text="Export Compact Tileset + JSON", command=self.export)
        btn_export.grid(row=0, column=11, padx=4, pady=4, sticky="w")

        # Info line
        self.info = tk.StringVar(value="Load a PNG to begin.")
        tk.Label(top, textvariable=self.info, fg="#444").grid(row=1, column=0, columnspan=12, sticky="w", pady=(4,0))

        # Canvas area with scrollbars
        mid = tk.Frame(self.root)
        mid.pack(side=tk.TOP, fill=tk.BOTH, expand=True)

        self.canvas = tk.Canvas(mid, bg="black")
        self.canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        vbar = tk.Scrollbar(mid, orient=tk.VERTICAL, command=self.canvas.yview)
        vbar.pack(side=tk.RIGHT, fill=tk.Y)
        hbar = tk.Scrollbar(self.root, orient=tk.HORIZONTAL, command=self.canvas.xview)
        hbar.pack(side=tk.BOTTOM, fill=tk.X)

        self.canvas.configure(yscrollcommand=vbar.set, xscrollcommand=hbar.set)

        # Mouse bindings
        self.canvas.bind("<Button-1>", self.on_click)
        self.canvas.bind("<B1-Motion>", self.on_drag)
        self.canvas.bind("<Button-3>", self.on_right_click)

        # Wheel zoom (Windows/Mac)
        self.canvas.bind("<MouseWheel>", self.on_wheel)          # Windows
        self.canvas.bind("<Button-4>", self.on_wheel_linux)      # Linux up
        self.canvas.bind("<Button-5>", self.on_wheel_linux)      # Linux down

        # Panning with middle mouse
        self.canvas.bind("<ButtonPress-2>", self.on_pan_start)
        self.canvas.bind("<B2-Motion>", self.on_pan_move)

        # Selection mode
        self.drag_select_mode = None  # "add" or "remove"
        self.is_panning = False
        self.pan_start = (0, 0)

    # ---------------------------
    # Image loading & drawing
    # ---------------------------

    def load_image(self):
        path = filedialog.askopenfilename(
            title="Select tileset PNG",
            filetypes=[("PNG Images", "*.png"), ("All files", "*.*")]
        )
        if not path:
            return

        try:
            img = Image.open(path).convert("RGBA")
        except Exception as e:
            messagebox.showerror("Error", f"Failed to open image:\n{e}")
            return

        self.img_path = path
        self.img = img
        self.img_w, self.img_h = img.size
        self.selected.clear()

        self.redraw()

    def redraw(self):
        if self.img is None:
            return

        tw = self.tile_w.get()
        th = self.tile_h.get()
        z = self.zoom.get()

        if tw <= 0 or th <= 0:
            messagebox.showerror("Invalid tile size", "Tile width/height must be > 0")
            return
        if z <= 0:
            messagebox.showerror("Invalid zoom", "Zoom must be > 0")
            return

        # Compute grid size in tiles
        self.tiles_x = self.img_w // tw
        self.tiles_y = self.img_h // th

        # Render scaled image for display
        disp_w = int(self.img_w * z)
        disp_h = int(self.img_h * z)
        disp = self.img.resize((disp_w, disp_h), Image.NEAREST)
        self.tk_img = ImageTk.PhotoImage(disp)

        self.canvas.delete("all")
        self.canvas_img_id = self.canvas.create_image(0, 0, anchor="nw", image=self.tk_img)

        # Scroll region
        self.canvas.config(scrollregion=(0, 0, disp_w, disp_h))

        # Draw grid + selection overlay
        self.draw_grid()
        self.draw_selection()

        self.info.set(
            f"{os.path.basename(self.img_path)} | "
            f"Image: {self.img_w}x{self.img_h}px | "
            f"Tile: {tw}x{th}px | "
            f"Grid: {self.tiles_x}x{self.tiles_y} tiles | "
            f"Selected: {len(self.selected)}"
        )

    def draw_grid(self):
        if self.img is None:
            return
        tw = self.tile_w.get()
        th = self.tile_h.get()
        z = self.zoom.get()

        # Draw grid lines (thin gray)
        color = "#303030"
        for x in range(self.tiles_x + 1):
            px = int(x * tw * z)
            self.canvas.create_line(px, 0, px, int(self.img_h * z), fill=color, tags="grid")
        for y in range(self.tiles_y + 1):
            py = int(y * th * z)
            self.canvas.create_line(0, py, int(self.img_w * z), py, fill=color, tags="grid")

    def draw_selection(self):
        if self.img is None:
            return
        tw = self.tile_w.get()
        th = self.tile_h.get()
        z = self.zoom.get()

        self.canvas.delete("sel")

        # Draw translucent-ish rectangles by drawing outline + stipple fill (Tk limitation)
        for (tx, ty) in self.selected:
            x0 = int(tx * tw * z)
            y0 = int(ty * th * z)
            x1 = int((tx + 1) * tw * z)
            y1 = int((ty + 1) * th * z)
            self.canvas.create_rectangle(
                x0, y0, x1, y1,
                outline="#00ff7f",
                width=2,
                fill="#00ff7f",
                stipple="gray25",
                tags="sel"
            )

    # ---------------------------
    # Picking logic
    # ---------------------------

    def canvas_to_tile(self, cx, cy):
        """Convert canvas coords -> tile coords in source grid."""
        if self.img is None:
            return None
        tw = self.tile_w.get()
        th = self.tile_h.get()
        z = self.zoom.get()

        # Account for scrolling
        x = self.canvas.canvasx(cx)
        y = self.canvas.canvasy(cy)

        tx = int(x // (tw * z))
        ty = int(y // (th * z))
        if tx < 0 or ty < 0 or tx >= self.tiles_x or ty >= self.tiles_y:
            return None
        return (tx, ty)

    def toggle_tile(self, t):
        if t in self.selected:
            self.selected.remove(t)
        else:
            self.selected.add(t)

    def set_tile(self, t, add=True):
        if add:
            self.selected.add(t)
        else:
            self.selected.discard(t)

    def on_click(self, e):
        t = self.canvas_to_tile(e.x, e.y)
        if not t:
            return
        # Left click toggles; drag will lock mode
        if t in self.selected:
            self.drag_select_mode = "remove"
            self.set_tile(t, add=False)
        else:
            self.drag_select_mode = "add"
            self.set_tile(t, add=True)

        self.draw_selection()
        self.info.set(self.info.get().rsplit("Selected:", 1)[0] + f"Selected: {len(self.selected)}")

    def on_drag(self, e):
        if self.drag_select_mode is None:
            return
        t = self.canvas_to_tile(e.x, e.y)
        if not t:
            return
        self.set_tile(t, add=(self.drag_select_mode == "add"))
        self.draw_selection()
        self.info.set(self.info.get().rsplit("Selected:", 1)[0] + f"Selected: {len(self.selected)}")

    def on_right_click(self, e):
        # Right click: toggle too (handy for one-off)
        t = self.canvas_to_tile(e.x, e.y)
        if not t:
            return
        self.toggle_tile(t)
        self.draw_selection()
        self.info.set(self.info.get().rsplit("Selected:", 1)[0] + f"Selected: {len(self.selected)}")

    def clear_selection(self):
        self.selected.clear()
        self.draw_selection()
        if self.img_path:
            self.info.set(self.info.get().rsplit("Selected:", 1)[0] + "Selected: 0")

    # ---------------------------
    # Zoom & Pan
    # ---------------------------

    def on_wheel(self, e):
        # Ctrl+wheel = zoom, otherwise scroll
        if (e.state & 0x0004) != 0:  # Ctrl key
            delta = 1 if e.delta > 0 else -1
            self._zoom_by(delta)
        else:
            # vertical scroll
            self.canvas.yview_scroll(-1 * (e.delta // 120), "units")

    def on_wheel_linux(self, e):
        # Linux wheel events are Button-4 / Button-5
        if e.num == 4:
            self._zoom_by(+1)
        elif e.num == 5:
            self._zoom_by(-1)

    def _zoom_by(self, delta_steps):
        z = self.zoom.get()
        # gentle scaling
        if delta_steps > 0:
            z *= 1.1
        else:
            z /= 1.1
        z = clamp(z, 0.25, 12.0)
        self.zoom.set(z)
        self.redraw()

    def on_pan_start(self, e):
        self.is_panning = True
        self.pan_start = (e.x, e.y)

    def on_pan_move(self, e):
        if not self.is_panning:
            return
        dx = self.pan_start[0] - e.x
        dy = self.pan_start[1] - e.y
        self.pan_start = (e.x, e.y)
        self.canvas.xview_scroll(int(dx), "units")
        self.canvas.yview_scroll(int(dy), "units")

    # ---------------------------
    # Export
    # ---------------------------

    def export(self):
        if self.img is None:
            messagebox.showerror("No image", "Load a tileset first.")
            return
        if not self.selected:
            messagebox.showerror("No selection", "Select at least one tile to export.")
            return

        tw = self.tile_w.get()
        th = self.tile_h.get()
        if tw <= 0 or th <= 0:
            messagebox.showerror("Invalid tile size", "Tile width/height must be > 0")
            return

        # Ask save path for PNG
        default_name = "compact_tileset.png"
        out_png = filedialog.asksaveasfilename(
            title="Save compact tileset PNG",
            defaultextension=".png",
            initialfile=default_name,
            filetypes=[("PNG Images", "*.png")]
        )
        if not out_png:
            return

        # Save JSON next to it
        out_json = os.path.splitext(out_png)[0] + ".json"

        # Build list in a stable order: row-major by (ty, tx)
        selected_sorted = sorted(self.selected, key=lambda t: (t[1], t[0]))
        n = len(selected_sorted)

        cols_pref = self.out_cols.get()
        cols, rows = compute_output_grid(n, cols_pref if cols_pref > 0 else None)

        out_img = Image.new("RGBA", (cols * tw, rows * th), (0, 0, 0, 0))

        mapping = {
            "source_image": os.path.basename(self.img_path) if self.img_path else None,
            "tile_size": {"w": tw, "h": th},
            "source_grid": {"cols": self.tiles_x, "rows": self.tiles_y},
            "output_grid": {"cols": cols, "rows": rows},
            "tiles": []
        }

        for i, (sx, sy) in enumerate(selected_sorted):
            # crop tile from source
            box = (sx * tw, sy * th, (sx + 1) * tw, (sy + 1) * th)
            tile = self.img.crop(box)

            dx = i % cols
            dy = i // cols
            out_img.paste(tile, (dx * tw, dy * th))

            mapping["tiles"].append({
                "src": {"x": sx, "y": sy, "index": sy * self.tiles_x + sx},
                "dst": {"x": dx, "y": dy, "index": i}
            })

        try:
            out_img.save(out_png)
            with open(out_json, "w", encoding="utf-8") as f:
                json.dump(mapping, f, indent=2)
        except Exception as e:
            messagebox.showerror("Export error", f"Failed to export:\n{e}")
            return

        messagebox.showinfo(
            "Export complete",
            f"Saved:\n- {out_png}\n- {out_json}\n\nTiles exported: {n}"
        )

# ---------------------------
# Run
# ---------------------------

if __name__ == "__main__":
    root = tk.Tk()
    root.geometry("1200x800")
    app = TilesetPickerApp(root)
    root.mainloop()
