const stageNameEl = document.getElementById("stageName");
const buildItemsEl = document.getElementById("buildItems");

async function fetchBuildStatus() {
  try {
    const response = await fetch("http://127.0.0.1:3333/api/build-status");
    if (response.ok) {
      const data = await response.json();
      updateBuildStatus(data);
    }
  } catch (error) {
    console.error("Failed to fetch build status:", error);
  }
}

function updateBuildStatus(data) {
  stageNameEl.textContent = `Stage ${data.stage_index + 1}: ${data.stage_name}`;

  if (data.items.length === 0) {
    buildItemsEl.innerHTML =
      '<li class="build-item">No build items in current stage</li>';
  } else {
    // Sort items alphabetically by unit name
    const sortedItems = [...data.items].sort((a, b) =>
      a.unit_name.localeCompare(b.unit_name),
    );

    buildItemsEl.innerHTML = sortedItems
      .map((item) => {
        let itemClass = "build-item";
        if (item.status.includes("Complete")) {
          itemClass += " complete";
        } else if (item.status.includes("Ready to build")) {
          itemClass += " ready";
        } else {
          itemClass += " waiting";
        }

        return `
      <li class="${itemClass}">
        <span class="unit-name">${item.unit_name}</span>
        <span class="unit-status">${item.status}</span>
      </li>
    `;
      })
      .join("");
  }

  // Upgrades
  const upgradeItemsEl = document.getElementById("upgradeItems");
  if (data.upgrades && data.upgrades.length > 0) {
    // Sort upgrades alphabetically
    const sortedUpgrades = [...data.upgrades].sort((a, b) =>
      a.upgrade_name.localeCompare(b.upgrade_name),
    );
    upgradeItemsEl.innerHTML = sortedUpgrades
      .map((upgrade) => {
        let itemClass = "build-item";
        if (upgrade.status.includes("Complete")) {
          itemClass += " complete";
        } else if (upgrade.status.includes("Ready")) {
          itemClass += " ready";
        } else {
          itemClass += " waiting";
        }
        return `
          <li class="${itemClass}">
            <span class="unit-name">${upgrade.upgrade_name}</span>
            <span class="unit-status">${upgrade.status}</span>
          </li>
        `;
      })
      .join("");
  } else {
    upgradeItemsEl.innerHTML =
      '<li class="build-item">No upgrades in current stage</li>';
  }
}

fetchBuildStatus();
setInterval(fetchBuildStatus, 1000);
