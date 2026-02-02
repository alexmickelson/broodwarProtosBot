import { PIXEL_TO_DISPLAY_SCALE, unitsGroup } from "./constants.js";

// Render units on the map
export function renderUnits(units) {
  if (!unitsGroup) return;

  // Create a set of current unit IDs
  const currentUnitIds = new Set(units.map((unit) => unit.unit_id));

  // Remove units that no longer exist
  const existingElements = unitsGroup.querySelectorAll("[data-unit-id]");
  existingElements.forEach((element) => {
    const unitId = parseInt(element.getAttribute("data-unit-id"));
    if (!currentUnitIds.has(unitId)) {
      element.remove();
    }
  });

  // Update or create units
  units.forEach((unit) => {
    const unitId = unit.unit_id;
    const x = unit.pixel_position.x * PIXEL_TO_DISPLAY_SCALE;
    const y = unit.pixel_position.y * PIXEL_TO_DISPLAY_SCALE;
    const width = unit.unit_width * PIXEL_TO_DISPLAY_SCALE;
    const height = unit.unit_height * PIXEL_TO_DISPLAY_SCALE;

    // Try to find existing elements for this unit
    let line = unitsGroup.querySelector(`line[data-unit-id="${unitId}"]`);
    let rect = unitsGroup.querySelector(`rect[data-unit-id="${unitId}"]`);

    // Handle target line
    if (unit.target_pixel_position) {
      const targetX = unit.target_pixel_position.x * PIXEL_TO_DISPLAY_SCALE;
      const targetY = unit.target_pixel_position.y * PIXEL_TO_DISPLAY_SCALE;

      if (!line) {
        line = document.createElementNS("http://www.w3.org/2000/svg", "line");
        line.setAttribute("data-unit-id", unitId);
        line.classList.add("unit-target-line");
        unitsGroup.appendChild(line);
      }

      line.setAttribute("x1", x);
      line.setAttribute("y1", y);
      line.setAttribute("x2", targetX);
      line.setAttribute("y2", targetY);
    } else if (line) {
      // Remove line if target no longer exists
      line.remove();
    }

    // Handle unit rectangle
    if (!rect) {
      rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
      rect.setAttribute("data-unit-id", unitId);
      rect.classList.add("unit-rect");

      // Add tooltip
      const title = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "title",
      );
      rect.appendChild(title);

      unitsGroup.appendChild(rect);
    }

    // Update rect attributes
    rect.setAttribute("x", x - width / 2);
    rect.setAttribute("y", y - height / 2);
    rect.setAttribute("width", width);
    rect.setAttribute("height", height);
    rect.setAttribute("fill", getUnitColor(unit.player_id, unit.player_name));

    // Update tooltip
    const title = rect.querySelector("title");
    if (title) {
      title.textContent = `${unit.unit_type}\nID: ${unit.unit_id}\nPlayer: ${unit.player_name || "Unknown"}`;
    }
  });
}

function getUnitColor(playerId, playerName) {
  // Check if player is Neutral
  if (playerName === "Neutral") {
    return "var(--color-neutral)";
  }

  // Color units based on player ID
  const colors = [
    "var(--color-player-red)",
    "var(--color-player-blue)",
    "var(--color-player-teal)",
    "var(--color-player-orange)",
    "var(--color-player-purple)",
    "var(--color-player-yellow)",
    "var(--color-player-brown)",
    "var(--color-player-white)",
  ];

  if (playerId !== null && playerId >= 0 && playerId < colors.length) {
    return colors[playerId];
  }
  return "var(--color-unit-unknown)"; // Gray for unknown/neutral
}

export async function fetchUnits() {
  try {
    const response = await fetch("http://127.0.0.1:3333/api/unit-info");
    if (response.ok) {
      const units = await response.json();
      renderUnits(units);
    }
  } catch (error) {
    console.error("Failed to fetch unit info:", error);
  }
}
