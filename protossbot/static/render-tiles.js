import { TILE_SIZE } from "./constants.js";

// Render map tiles
export function renderMapTiles(mapData, contentGroup) {
  const tileMap = new Map();
  mapData.tiles.forEach((tile) => {
    const key = `${tile.x},${tile.y}`;
    tileMap.set(key, tile);
  });

  // Render all tiles
  for (let y = 0; y < mapData.map_height; y++) {
    for (let x = 0; x < mapData.map_width; x++) {
      const key = `${x},${y}`;
      const tile = tileMap.get(key);

      const rect = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "rect",
      );
      rect.setAttribute("x", x * TILE_SIZE);
      rect.setAttribute("y", y * TILE_SIZE);
      rect.setAttribute("width", TILE_SIZE);
      rect.setAttribute("height", TILE_SIZE);

      // Color based on walkability
      if (tile && tile.is_walkable) {
        rect.setAttribute("fill", "var(--color-tile-walkable)");
      } else {
        rect.setAttribute("fill", "var(--color-tile-unwalkable)");
      }

      contentGroup.appendChild(rect);
    }
  }
}
