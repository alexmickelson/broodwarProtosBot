import { TILE_SIZE, baseLocationsGroup } from "./constants.js";

// Render base locations on the map
export function renderBaseLocations(baseLocations) {
  if (!baseLocationsGroup) return;

  // Clear existing base location elements
  baseLocationsGroup.innerHTML = "";

  baseLocations.forEach((base) => {
    const baseX = base.position.x * TILE_SIZE;
    const baseY = base.position.y * TILE_SIZE;

    // Draw path to location
    if (base.path_to_location && base.path_to_location.length > 1) {
      const pathData = base.path_to_location
        .map((tile, index) => {
          const x = tile.x * TILE_SIZE + TILE_SIZE / 2;
          const y = tile.y * TILE_SIZE + TILE_SIZE / 2;
          return index === 0 ? `M ${x} ${y}` : `L ${x} ${y}`;
        })
        .join(" ");

      const path = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "path",
      );
      path.setAttribute("d", pathData);
      path.classList.add("base-path");
      baseLocationsGroup.appendChild(path);
    }

    // Draw checked positions (mineral spots)
    base.checked_positions.forEach((checked) => {
      const cx = checked.tile_position.x * TILE_SIZE;
      const cy = checked.tile_position.y * TILE_SIZE;

      const rect = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "rect",
      );
      rect.setAttribute("x", cx);
      rect.setAttribute("y", cy);
      rect.setAttribute("width", TILE_SIZE);
      rect.setAttribute("height", TILE_SIZE);
      rect.classList.add(
        checked.is_valid ? "checked-valid" : "checked-invalid",
      );
      baseLocationsGroup.appendChild(rect);
    });

    // Draw base box (4x3 tiles for command center size)
    const baseBox = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "rect",
    );
    baseBox.setAttribute("x", baseX);
    baseBox.setAttribute("y", baseY);
    baseBox.setAttribute("width", TILE_SIZE * 4);
    baseBox.setAttribute("height", TILE_SIZE * 3);
    baseBox.classList.add("base-box");
    baseLocationsGroup.appendChild(baseBox);

    // Draw base ID text
    const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
    text.setAttribute("x", baseX + TILE_SIZE * 2);
    text.setAttribute("y", baseY + TILE_SIZE * 1.5);
    text.setAttribute("text-anchor", "middle");
    text.setAttribute("dominant-baseline", "middle");
    text.classList.add("base-id-text");
    text.textContent = `Base ${base.id}`;
    baseLocationsGroup.appendChild(text);
  });
}

export async function fetchBaseLocations() {
  try {
    const response = await fetch("http://127.0.0.1:3333/api/base-locations");
    if (response.ok) {
      const baseLocations = await response.json();
      renderBaseLocations(baseLocations);
    }
  } catch (error) {
    console.error("Failed to fetch base locations:", error);
  }
}
