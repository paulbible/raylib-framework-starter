import argparse
import json
import math
import os
import random
from dataclasses import dataclass
from typing import List, Tuple, Dict, Optional

import pygame

# ----------------------------
# CONFIG (tweak these freely)
# ----------------------------
DEFAULT_GRID_W = 80
DEFAULT_GRID_H = 60      
DEFAULT_SCALE = 3            # how big to draw tiles initially
PALETTE_COLS = 7            # tiles shown per row in palette view
FOV_RADIUS_TILES = 7         # player FOV radius
MAZE_CORRIDOR_WIDTH = 3      # hallway width in tiles (>=3 per your requirement)

# Choose "wall" and "floor" tile IDs from your tileset (index into atlas).
# You should adjust these to match your sheet.
DEFAULT_TILE_SIZE = 32       # pixels per tile in the tileset sheet
DEFAULT_WALL_TILE_ID = 92
DEFAULT_FLOOR_TILE_ID = 5

# Entities saved in JSON
ENTITY_TYPES = ["player", "goal", "tank", "shooter"]


@dataclass
class Entity:
    kind: str
    x: int
    y: int


def clamp(v, lo, hi):
    return max(lo, min(hi, v))


def load_image(path: str) -> pygame.Surface:
    img = pygame.image.load(path).convert_alpha()
    return img


def slice_tileset(sheet: pygame.Surface, tile_size: int) -> List[pygame.Surface]:
    tiles = []
    w, h = sheet.get_size()
    cols = w // tile_size
    rows = h // tile_size
    for y in range(rows):
        for x in range(cols):
            rect = pygame.Rect(x * tile_size, y * tile_size, tile_size, tile_size)
            tile = sheet.subsurface(rect).copy()
            tiles.append(tile)
    return tiles


def make_empty_grid(w: int, h: int, fill: int = -1) -> List[List[int]]:
    return [[fill for _ in range(w)] for _ in range(h)]


# ----------------------------
# MAZE GENERATION (DFS)
# corridors with width >= corridor_w
# ----------------------------
def generate_maze_tilegrid(tile_w: int, tile_h: int,
                           wall_id: int, floor_id: int,
                           corridor_w: int = 3,
                           cell_size: int = 2) -> List[List[int]]:
    """
    Generates a maze on a tile grid.

    Approach:
    - Work on a coarse "cell" grid (maze cells), carve passages with DFS.
    - Then "inflate" passages into corridors of width corridor_w on the tile grid.

    Notes:
    - tile_w/tile_h should be comfortably large.
    - corridor_w >= 3 as required.
    """
    corridor_w = max(1, corridor_w)

    # Coarse maze cells
    step = corridor_w + 1
    usable_w = tile_w - 2
    usable_h = tile_h - 2

    cw = max(2, (usable_w - corridor_w) // step + 1)
    ch = max(2, (usable_h - corridor_w) // step + 1)


    visited = [[False for _ in range(cw)] for _ in range(ch)]
    grid = [[wall_id for _ in range(tile_w)] for _ in range(tile_h)]

    def cell_to_tile(cx, cy) -> Tuple[int, int]:
        tx = 1 + cx * step
        ty = 1 + cy * step
        return tx, ty


    def carve_corridor(tx, ty):
        half = corridor_w // 2
        for yy in range(ty - half, ty - half + corridor_w):
            for xx in range(tx - half, tx - half + corridor_w):
                if 0 <= xx < tile_w and 0 <= yy < tile_h:
                    grid[yy][xx] = floor_id

    def carve_line(a: Tuple[int, int], b: Tuple[int, int]):
        # carve a "thick" line between two corridor centers
        ax, ay = a
        bx, by = b
        steps = max(abs(bx - ax), abs(by - ay))
        if steps == 0:
            carve_corridor(ax, ay)
            return
        for i in range(steps + 1):
            t = i / steps
            x = round(ax + (bx - ax) * t)
            y = round(ay + (by - ay) * t)
            carve_corridor(x, y)

    # DFS stack
    start = (random.randrange(cw), random.randrange(ch))
    stack = [start]
    visited[start[1]][start[0]] = True

    # carve start cell
    sx, sy = cell_to_tile(*start)
    carve_corridor(sx, sy)

    dirs = [(1,0), (-1,0), (0,1), (0,-1)]

    while stack:
        cx, cy = stack[-1]

        neighbors = []
        for dx, dy in dirs:
            nx, ny = cx + dx, cy + dy
            if 0 <= nx < cw and 0 <= ny < ch and not visited[ny][nx]:
                neighbors.append((nx, ny))

        if not neighbors:
            stack.pop()
            continue

        nx, ny = random.choice(neighbors)
        visited[ny][nx] = True

        a = cell_to_tile(cx, cy)
        b = cell_to_tile(nx, ny)

        # carve connection
        carve_line(a, b)
        carve_corridor(*b)

        stack.append((nx, ny))

    # optional: add a few loops to make dodging/shooting more interesting
    for _ in range((cw * ch) // 12):
        cx = random.randrange(cw)
        cy = random.randrange(ch)
        dx, dy = random.choice(dirs)
        nx, ny = cx + dx, cy + dy
        if 0 <= nx < cw and 0 <= ny < ch:
            carve_line(cell_to_tile(cx, cy), cell_to_tile(nx, ny))

    # Keep borders walls
    for x in range(tile_w):
        grid[0][x] = wall_id
        grid[tile_h-1][x] = wall_id
    for y in range(tile_h):
        grid[y][0] = wall_id
        grid[y][tile_w-1] = wall_id

    return grid


# ----------------------------
# EDITOR
# ----------------------------
class Editor:
    def __init__(self, tileset_path: str, tile_size: int, grid_w: int, grid_h: int):
        pygame.init()
        pygame.display.set_caption("Dungeon Editor (raylib/Rust)")

        self.screen = pygame.display.set_mode((1280, 800))
        self.clock = pygame.time.Clock()

        self.sheet = load_image(tileset_path)
        self.tiles = slice_tileset(self.sheet, tile_size)
        self.tile_size = tile_size

        self.grid_w = grid_w
        self.grid_h = grid_h
        self.grid = make_empty_grid(grid_w, grid_h, fill=-1)

        self.entities: List[Entity] = []

        self.wall_tile_id = DEFAULT_WALL_TILE_ID
        self.floor_tile_id = DEFAULT_FLOOR_TILE_ID

        self.scale = float(DEFAULT_SCALE)  

        self.cam_x = 0
        self.cam_y = 0

        self.selected_tile_id = 0
        self.selected_place_mode: Optional[str] = None  # "player"/"goal"/"tank"/"shooter"/None

        self.show_fov = True

        self.save_path = "map.json"

        # palette / input state
        self.palette_panel_w = 520
        self.palette_cols = PALETTE_COLS
        self.palette_rows = 9
        self.palette_tile_draw = self.tile_size * 2  # palette tile size in pixels

        self.eyedropper = False  # press E, then click map to pick a tile
        self.is_panning = False
        self.last_mouse = (0, 0)

         # palette paging
        self.palette_page = 0

        # place a default player to make FOV meaningful
        self.set_unique_entity("player", grid_w // 2, grid_h // 2)

        # help panel
        self.show_help = False

        self._consume_left_click = False



    def set_unique_entity(self, kind: str, x: int, y: int):
        # remove existing of that kind
        self.entities = [e for e in self.entities if e.kind != kind]
        self.entities.append(Entity(kind, x, y))

    def add_entity(self, kind: str, x: int, y: int):
        # allow multiple enemies, but player/goal unique
        if kind in ("player", "goal"):
            self.set_unique_entity(kind, x, y)
        else:
            self.entities.append(Entity(kind, x, y))

    def get_player_pos(self) -> Tuple[int, int]:
        for e in self.entities:
            if e.kind == "player":
                return e.x, e.y
        return (0, 0)

    def tile_to_screen(self, tx: int, ty: int) -> Tuple[int, int]:
        ts = max(1, int(self.tile_size * self.scale))

        sx = tx * ts + self.cam_x
        sy = ty * ts + self.cam_y
        return sx, sy

    def screen_to_tile(self, sx: int, sy: int) -> Tuple[int, int]:
        ts = max(1, int(self.tile_size * self.scale))

        tx = (sx - self.cam_x) // ts
        ty = (sy - self.cam_y) // ts
        return int(tx), int(ty)

    def in_bounds(self, tx: int, ty: int) -> bool:
        return 0 <= tx < self.grid_w and 0 <= ty < self.grid_h

    def _palette_layout(self):
        """Return palette geometry: (x0, y0, ts, cols, tiles_per_page, base, max_pages)."""
        panel_w = self.palette_panel_w
        x0 = self.screen.get_width() - panel_w

        font_line_h = 22
        info_lines = 6  # must match draw_palette()
        y0 = 10 + info_lines * font_line_h + 10 + 10

        ts = self.palette_tile_draw
        cols = self.palette_cols
        tiles_per_page = cols * self.palette_rows

        max_pages = max(1, math.ceil(len(self.tiles) / tiles_per_page))
        self.palette_page = clamp(self.palette_page, 0, max_pages - 1)

        base = self.palette_page * tiles_per_page
        return x0, y0, ts, cols, tiles_per_page, base, max_pages


    def _in_palette_panel(self, sx: int, sy: int) -> bool:
        x0 = self.screen.get_width() - self.palette_panel_w
        return sx >= x0

    def _palette_tile_at(self, sx: int, sy: int) -> Optional[int]:
        """If (sx, sy) hits a tile in the palette, return its tile id."""
        x0, y0, ts, cols, tiles_per_page, base, max_pages = self._palette_layout()

        gx = sx - (x0 + 10)
        gy = sy - y0
        if gx < 0 or gy < 0:
            return None
        col = gx // (ts + 2)
        row = gy // (ts + 2)
        if col < 0 or col >= cols or row < 0 or row >= self.palette_rows:
            return None
        tid = base + int(row) * cols + int(col)
        if 0 <= tid < len(self.tiles):
            rx = gx % (ts + 2)
            ry = gy % (ts + 2)
            if rx <= ts and ry <= ts:
                return tid
        return None

    def draw_grid(self):
        ts = max(1, int(self.tile_size * self.scale))


        # Determine visible tile range
        min_tx = clamp((-self.cam_x) // ts - 1, 0, self.grid_w)
        min_ty = clamp((-self.cam_y) // ts - 1, 0, self.grid_h)
        max_tx = clamp(((-self.cam_x) + self.screen.get_width()) // ts + 2, 0, self.grid_w)
        max_ty = clamp(((-self.cam_y) + self.screen.get_height()) // ts + 2, 0, self.grid_h)

        for ty in range(min_ty, max_ty):
            for tx in range(min_tx, max_tx):
                tid = self.grid[ty][tx]
                sx, sy = self.tile_to_screen(tx, ty)

                if tid >= 0 and tid < len(self.tiles):
                    tile = pygame.transform.scale(self.tiles[tid], (ts, ts))
                    self.screen.blit(tile, (sx, sy))
                else:
                    # faint background for empty
                    pygame.draw.rect(self.screen, (25, 25, 25), pygame.Rect(sx, sy, ts, ts))

                # grid lines
                pygame.draw.rect(self.screen, (40, 40, 40), pygame.Rect(sx, sy, ts, ts), 1)

    def draw_entities(self):
        ts = max(1, int(self.tile_size * self.scale))

        font = pygame.font.SysFont(None, 20)

        for e in self.entities:
            sx, sy = self.tile_to_screen(e.x, e.y)
            rect = pygame.Rect(sx, sy, ts, ts)

            # simple colored markers (editor-only)
            if e.kind == "player":
                color = (80, 200, 255)
                label = "P"
            elif e.kind == "goal":
                color = (255, 220, 80)
                label = "G"
            elif e.kind == "tank":
                color = (255, 100, 100)
                label = "T"
            else:  # shooter
                color = (255, 140, 255)
                label = "S"

            pygame.draw.rect(self.screen, color, rect, 3)
            txt = font.render(label, True, color)
            self.screen.blit(txt, (sx + 4, sy + 2))

    def draw_fov_overlay(self):
        if not self.show_fov:
            return

        px, py = self.get_player_pos()
        ts = max(1, int(self.tile_size * self.scale))


        # circular radius
        r = FOV_RADIUS_TILES
        for y in range(py - r, py + r + 1):
            for x in range(px - r, px + r + 1):
                if not self.in_bounds(x, y):
                    continue
                if (x - px) * (x - px) + (y - py) * (y - py) <= r * r:
                    sx, sy = self.tile_to_screen(x, y)
                    # translucent overlay
                    overlay = pygame.Surface((ts, ts), pygame.SRCALPHA)
                    overlay.fill((80, 180, 255, 35))
                    self.screen.blit(overlay, (sx, sy))

        # darken outside FOV a bit (simple)
        # (Cheap approach: draw a translucent black over whole screen, then redraw FOV tiles)
        # Keeping simple: skip global darkening. Your game can do real masking.


    def draw_palette(self):
        panel_w = self.palette_panel_w
        x0 = self.screen.get_width() - panel_w
        pygame.draw.rect(self.screen, (15, 15, 15), pygame.Rect(x0, 0, panel_w, self.screen.get_height()))

        font = pygame.font.SysFont(None, 22)
        
        # Help button
        help_rect = pygame.Rect(x0 + panel_w - 90, 10, 80, 30)
        pygame.draw.rect(self.screen, (35, 35, 35), help_rect, border_radius=6)
        pygame.draw.rect(self.screen, (80, 80, 80), help_rect, 2, border_radius=6)
        self.screen.blit(font.render("Help", True, (230, 230, 230)), (help_rect.x + 18, help_rect.y + 6))
        self._help_rect = help_rect

        ts = self.palette_tile_draw
        cols = self.palette_cols
        start_y = 10

        mode = self.selected_place_mode or "paint"
        if self.eyedropper:
            mode = "eyedropper"

        info = [
            f"Selected tile: {self.selected_tile_id}",
            f"Mode: {mode}",
            f"Scale: {self.scale}x",
            "Controls:",
            "  WASD/Arrows: pan | MMB drag: pan | Wheel: zoom",
            "  Ctrl+S save | Ctrl+O load | [ ] step tile | E pick tile",
        ]
        y = start_y
        for line in info:
            t = font.render(line, True, (220, 220, 220))
            self.screen.blit(t, (x0 + 10, y))
            y += 22

        y += 10

        # tiles_per_page = cols * self.palette_rows
        # page = self.selected_tile_id // tiles_per_page
        # base = page * tiles_per_page
        x0, y0, ts, cols, tiles_per_page, base, max_pages = self._palette_layout()


        for i in range(tiles_per_page):
            tid = base + i
            if tid >= len(self.tiles):
                break
            row = i // cols
            col = i % cols
            px = x0 + 10 + col * (ts + 2)
            py = y + row * (ts + 2)

            tile = pygame.transform.scale(self.tiles[tid], (ts, ts))
            self.screen.blit(tile, (px, py))
            if tid == self.selected_tile_id:
                pygame.draw.rect(self.screen, (255, 255, 255), pygame.Rect(px, py, ts, ts), 2)

        # page_txt = font.render(f"Page {page+1}", True, (180, 180, 180))
        # self.screen.blit(page_txt, (x0 + 10, self.screen.get_height() - 26))
                # Page display + buttons
        btn_y = self.screen.get_height() - 46
        btn_h = 30
        btn_w = 52

        prev_rect = pygame.Rect(x0 + 10, btn_y, btn_w, btn_h)
        next_rect = pygame.Rect(x0 + 10 + btn_w + 8, btn_y, btn_w, btn_h)

        pygame.draw.rect(self.screen, (35, 35, 35), prev_rect, border_radius=6)
        pygame.draw.rect(self.screen, (35, 35, 35), next_rect, border_radius=6)
        pygame.draw.rect(self.screen, (80, 80, 80), prev_rect, 2, border_radius=6)
        pygame.draw.rect(self.screen, (80, 80, 80), next_rect, 2, border_radius=6)

        prev_txt = font.render("<<", True, (230, 230, 230))
        next_txt = font.render(">>", True, (230, 230, 230))
        self.screen.blit(prev_txt, (prev_rect.x + 14, prev_rect.y + 6))
        self.screen.blit(next_txt, (next_rect.x + 14, next_rect.y + 6))

        page_txt = font.render(f"Page {self.palette_page + 1} / {max_pages}", True, (180, 180, 180))
        self.screen.blit(page_txt, (x0 + 10 + 2 * (btn_w + 8), btn_y + 6))

        # store for click handling
        self._palette_prev_rect = prev_rect
        self._palette_next_rect = next_rect

    def _page_for_tile(self, tile_id: int) -> int:
        _, _, _, cols, tiles_per_page, _, max_pages = self._palette_layout()
        return clamp(tile_id // tiles_per_page, 0, max_pages - 1)

    def palette_next_page(self):
        _, _, _, _, _, _, max_pages = self._palette_layout()
        self.palette_page = (self.palette_page + 1) % max_pages

    def palette_prev_page(self):
        _, _, _, _, _, _, max_pages = self._palette_layout()
        self.palette_page = (self.palette_page - 1) % max_pages

    def save_json(self):
        data = {
            "version": 1,
            "grid_w": self.grid_w,
            "grid_h": self.grid_h,
            "tile_size_px": self.tile_size,
            "tiles": self.grid,  # 2D array of tile IDs (-1 empty)
            "entities": [e.__dict__ for e in self.entities],
            "meta": {
                "notes": "Load this JSON in Rust. Use tiles[y][x] to draw. Entities list for spawns.",
                "fov_radius_tiles": FOV_RADIUS_TILES,
            }
        }
        with open(self.save_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)
        print(f"[Saved] {self.save_path}")

    def load_json(self):
        if not os.path.exists(self.save_path):
            print(f"[Load] No file found: {self.save_path}")
            return
        with open(self.save_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        self.grid_w = int(data["grid_w"])
        self.grid_h = int(data["grid_h"])
        self.grid = data["tiles"]
        self.entities = [Entity(**e) for e in data.get("entities", [])]
        print(f"[Loaded] {self.save_path}")

    def prompt_maze(self):
        print("Maze generation:")
        try:
            w = int(input(f"Grid width tiles (current {self.grid_w}): ").strip() or self.grid_w)
            h = int(input(f"Grid height tiles (current {self.grid_h}): ").strip() or self.grid_h)
            corridor = int(input(f"Corridor width tiles (>=3 recommended, default {MAZE_CORRIDOR_WIDTH}): ").strip() or MAZE_CORRIDOR_WIDTH)
            wall = int(input(f"Wall tile id (default {self.wall_tile_id}): ").strip() or self.wall_tile_id)
            floor = int(input(f"Floor tile id (default {self.floor_tile_id}): ").strip() or self.floor_tile_id)
        except Exception:
            print("Invalid input; using defaults.")
            w, h, corridor, wall, floor = self.grid_w, self.grid_h, MAZE_CORRIDOR_WIDTH, self.wall_tile_id, self.floor_tile_id

        self.grid_w, self.grid_h = w, h
        self.grid = generate_maze_tilegrid(w, h, wall, floor, corridor_w=corridor)

        # Keep player/goal inside bounds
        px, py = self.get_player_pos()
        self.set_unique_entity("player", clamp(px, 1, w-2), clamp(py, 1, h-2))
        self.set_unique_entity("goal", w-2, h-2)

        print("[Maze] Generated.")

    def handle_mouse_paint(self):
        mx, my = pygame.mouse.get_pos()
        buttons = pygame.mouse.get_pressed(3)

        # Palette interactions are handled on MOUSEBUTTONDOWN to avoid repeats.
        # If we just clicked in the palette this frame, ignore painting.
        if self._in_palette_panel(mx, my) or self._consume_left_click:
            return

    # def handle_mouse_paint(self):
    #     mx, my = pygame.mouse.get_pos()
    #     buttons = pygame.mouse.get_pressed(3)

    #     if self._in_palette_panel(mx, my):
    #         if buttons[0]:
    #             if hasattr(self, "_help_rect") and self._help_rect.collidepoint(mx, my):
    #                 self.show_help = not self.show_help
    #                 return

    #             # page buttons
    #             if hasattr(self, "_palette_prev_rect") and self._palette_prev_rect.collidepoint(mx, my):
    #                 self.palette_prev_page()
    #                 return
    #             if hasattr(self, "_palette_next_rect") and self._palette_next_rect.collidepoint(mx, my):
    #                 self.palette_next_page()
    #                 return

    #             # palette tile click selects tile
    #             tid = self._palette_tile_at(mx, my)
    #             if tid is not None:
    #                 self.selected_tile_id = tid
    #         return


    #     # Palette click selects tile
    #     if self._in_palette_panel(mx, my):
    #         if buttons[0]:
    #             tid = self._palette_tile_at(mx, my)
    #             if tid is not None:
    #                 self.selected_tile_id = tid
    #         return

        tx, ty = self.screen_to_tile(mx, my)
        if not self.in_bounds(tx, ty):
            return

        # Eyedropper: pick tile from map then exit eyedropper mode
        if self.eyedropper and buttons[0]:
            tid = self.grid[ty][tx]
            if tid is not None and tid >= 0:
                self.selected_tile_id = tid
                self.palette_page = self._page_for_tile(self.selected_tile_id)
            self.eyedropper = False                
            return

        # Normal painting / entity placement
        if buttons[0]:  # left paint
            if self.selected_place_mode:
                self.add_entity(self.selected_place_mode, tx, ty)
            else:
                self.grid[ty][tx] = self.selected_tile_id
        elif buttons[2]:  # right erase
            if self.selected_place_mode:
                self.entities = [e for e in self.entities if not (e.x == tx and e.y == ty and e.kind == self.selected_place_mode)]
            else:
                self.grid[ty][tx] = -1

    def draw_help_overlay(self):
        if not self.show_help:
            return

        w, h = self.screen.get_size()
        overlay = pygame.Surface((w, h), pygame.SRCALPHA)
        overlay.fill((0, 0, 0, 160))
        self.screen.blit(overlay, (0, 0))

        font_title = pygame.font.SysFont(None, 42)
        font = pygame.font.SysFont(None, 26)

        box = pygame.Rect(80, 60, w - 160, h - 120)
        pygame.draw.rect(self.screen, (25, 25, 25), box, border_radius=12)
        pygame.draw.rect(self.screen, (120, 120, 120), box, 2, border_radius=12)

        y = box.y + 20
        self.screen.blit(font_title.render("MapMaker Help", True, (240, 240, 240)), (box.x + 20, y))
        y += 52

        lines = [
            "Pan: WASD / Arrow keys  |  Middle Mouse Drag",
            "Zoom: Mouse Wheel",
            "Paint tile: Left click/drag     Erase: Right click/drag",
            "Pick tile from map: E, then click a tile (eyedropper)",
            "Select tile from palette: Click palette tile",
            "Palette pages: PageUp/PageDown or , / .  (or click << >> buttons)",
            "Entities: 1 Player, 2 Goal, 3 Tank, 4 Shooter, 0 back to painting tiles",
            "Toggle FOV overlay: F",
            "Generate maze: G",
            "Save: Ctrl+S    Load: Ctrl+O",
            "Close help: H or F1",
        ]

        for line in lines:
            self.screen.blit(font.render(line, True, (230, 230, 230)), (box.x + 20, y))
            y += 30


    def run(self):
        running = True
        while running:
            self.clock.tick(60)

            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    running = False

                # if event.type == pygame.MOUSEWHEEL:
                #     self.scale = clamp(self.scale + event.y, 1, 8)
                if event.type == pygame.MOUSEWHEEL:
                    # smooth zoom: 10% per wheel notch
                    factor = 1.1 ** event.y
                    self.scale = clamp(self.scale * factor, 0.10, 12.0)  # allow far zoom-out


                if event.type == pygame.MOUSEBUTTONDOWN:
                    if event.button == 2:  # middle mouse
                        self.is_panning = True
                        self.last_mouse = pygame.mouse.get_pos()

                    if event.button == 1:  # LEFT mouse DOWN (single-trigger)
                        mx, my = event.pos

                        if self._in_palette_panel(mx, my):
                            # Help button
                            if hasattr(self, "_help_rect") and self._help_rect.collidepoint(mx, my):
                                self.show_help = not self.show_help
                                self._consume_left_click = True

                            # Page buttons
                            elif hasattr(self, "_palette_prev_rect") and self._palette_prev_rect.collidepoint(mx, my):
                                self.palette_prev_page()
                                self._consume_left_click = True

                            elif hasattr(self, "_palette_next_rect") and self._palette_next_rect.collidepoint(mx, my):
                                self.palette_next_page()
                                self._consume_left_click = True

                            else:
                                # Palette tile selection
                                tid = self._palette_tile_at(mx, my)
                                if tid is not None:
                                    self.selected_tile_id = tid
                                    self._consume_left_click = True


                if event.type == pygame.MOUSEBUTTONUP:
                    if event.button == 2:
                        self.is_panning = False
                    if event.button == 1:
                        self._consume_left_click = False


                if event.type == pygame.MOUSEMOTION:
                    if self.is_panning:
                        mx, my = pygame.mouse.get_pos()
                        lx, ly = self.last_mouse
                        dx, dy = mx - lx, my - ly
                        self.cam_x += dx
                        self.cam_y += dy
                        self.last_mouse = (mx, my)

                if event.type == pygame.KEYDOWN:
                    if event.key == pygame.K_ESCAPE:
                        running = False

                    mods = pygame.key.get_mods()
                    ctrl = mods & pygame.KMOD_CTRL

                    # Save/Load (no movement conflict)
                    if ctrl and event.key == pygame.K_s:
                        self.save_json()
                    if ctrl and event.key == pygame.K_o:
                        self.load_json()

                    # Tile stepping
                    # if event.key == pygame.K_LEFTBRACKET:
                    #     self.selected_tile_id = clamp(self.selected_tile_id - 1, 0, len(self.tiles) - 1)
                    # if event.key == pygame.K_RIGHTBRACKET:
                    #     self.selected_tile_id = clamp(self.selected_tile_id + 1, 0, len(self.tiles) - 1)
                    if event.key == pygame.K_LEFTBRACKET:
                        self.selected_tile_id = clamp(self.selected_tile_id - 1, 0, len(self.tiles) - 1)
                        self.palette_page = self._page_for_tile(self.selected_tile_id)

                    if event.key == pygame.K_RIGHTBRACKET:
                        self.selected_tile_id = clamp(self.selected_tile_id + 1, 0, len(self.tiles) - 1)
                        self.palette_page = self._page_for_tile(self.selected_tile_id)

                    # Eyedropper
                    if event.key == pygame.K_e:
                        self.eyedropper = True
                        self.selected_place_mode = None  # eyedropper is for tiles

                    # Entity modes
                    if event.key == pygame.K_1:
                        self.selected_place_mode = "player"
                        self.eyedropper = False
                    if event.key == pygame.K_2:
                        self.selected_place_mode = "goal"
                        self.eyedropper = False
                    if event.key == pygame.K_3:
                        self.selected_place_mode = "tank"
                        self.eyedropper = False
                    if event.key == pygame.K_4:
                        self.selected_place_mode = "shooter"
                        self.eyedropper = False
                    if event.key == pygame.K_0:
                        self.selected_place_mode = None
                        self.eyedropper = False

                    if event.key == pygame.K_f:
                        self.show_fov = not self.show_fov

                    if event.key == pygame.K_g:
                        self.prompt_maze()

                    # keep L as a convenience load key (optional)
                    if (not ctrl) and event.key == pygame.K_l:
                        self.load_json()

                    # Palette page navigation
                    if event.key in (pygame.K_PAGEUP, pygame.K_COMMA):
                        self.palette_prev_page()
                    if event.key in (pygame.K_PAGEDOWN, pygame.K_PERIOD):
                        self.palette_next_page()
                    
                    # Help panel
                    if event.key in (pygame.K_h, pygame.K_F1):
                        self.show_help = not self.show_help

                    # fit to screen
                    if event.key == pygame.K_z:
                        # fit whole map into view (rough)
                        w_px = self.grid_w * self.tile_size
                        h_px = self.grid_h * self.tile_size
                        sx = (self.screen.get_width() - self.palette_panel_w) / w_px
                        sy = self.screen.get_height() / h_px
                        self.scale = clamp(min(sx, sy), 0.05, 12.0)
                        self.cam_x = 0
                        self.cam_y = 0

            # keyboard camera panning (WASD + arrows)
            keys = pygame.key.get_pressed()
            pan_speed = 14
            if keys[pygame.K_a] or keys[pygame.K_LEFT]:
                self.cam_x += pan_speed
            if keys[pygame.K_d] or keys[pygame.K_RIGHT]:
                self.cam_x -= pan_speed
            if keys[pygame.K_w] or keys[pygame.K_UP]:
                self.cam_y += pan_speed
            if keys[pygame.K_s] or keys[pygame.K_DOWN]:
                self.cam_y -= pan_speed
            

            # painting 
            if not self.is_panning:  # avoid painting while panning with MMB
                self.handle_mouse_paint()

            # draw
            self.screen.fill((10, 10, 10))
            self.draw_grid()
            self.draw_fov_overlay()
            self.draw_entities()
            self.draw_palette()
            self.draw_help_overlay()
            pygame.display.flip()

        pygame.quit()

    



def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tileset", default="tileset0.png", help="Path to tileset PNG (default: tileset0.png next to script)")
    parser.add_argument("--tile_size", type=int, default=DEFAULT_TILE_SIZE, help="Tile size in pixels (usually 16)")
    parser.add_argument("--grid_w", type=int, default=DEFAULT_GRID_W)
    parser.add_argument("--grid_h", type=int, default=DEFAULT_GRID_H)
    parser.add_argument("--save", default="map.json", help="Save/load path (default: map.json)")
    args = parser.parse_args()

    tileset_path = args.tileset
    if not os.path.isabs(tileset_path):
        # make relative paths resolve from script directory
        script_dir = os.path.dirname(os.path.abspath(__file__))
        tileset_path = os.path.join(script_dir, tileset_path)

    if not os.path.exists(tileset_path):
        print(f"[Error] Tileset not found: {tileset_path}")
        print("Tip: pass --tileset with the full path, or put tileset0.png next to this script.")
        raise SystemExit(2)

    try:
        ed = Editor(tileset_path, args.tile_size, args.grid_w, args.grid_h)
        ed.save_path = args.save
        ed.run()
    except pygame.error as e:
        print(f"[Pygame error] {e}")
        raise


if __name__ == "__main__":
    main()
