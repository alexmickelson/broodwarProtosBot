const buildHistoryItemsEl = document.getElementById("buildHistoryItems");

async function fetchBuildHistory() {
  try {
    const response = await fetch("http://127.0.0.1:3333/api/build-history");
    if (response.ok) {
      const data = await response.json();
      updateBuildHistory(data);
    } else {
      buildHistoryItemsEl.innerHTML =
        '<li class="build-item">Failed to load build history</li>';
    }
  } catch (error) {
    buildHistoryItemsEl.innerHTML =
      '<li class="build-item">Error loading build history</li>';
  }
}

function updateBuildHistory(items) {
  if (!items || items.length === 0) {
    buildHistoryItemsEl.innerHTML =
      '<li class="build-item no-history-msg">No build history items</li>';
    return;
  }
  buildHistoryItemsEl.innerHTML = items
    .map(function (item) {
      var unitName = item.unit_name ? item.unit_name : "Unknown";
      var unitId =
        item.assigned_unit_id !== undefined && item.assigned_unit_id !== null
          ? item.assigned_unit_id
          : "N/A";
      return (
        '<li class="build-item">' +
        '<span class="unit-name">' +
        unitName +
        "</span>" +
        '<span class="unit-status">' +
        item.status +
        "</span>" +
        '<span class="unit-status">ID: ' +
        unitId +
        "</span>" +
        "</li>"
      );
    })
    .join("");
}

fetchBuildHistory();
setInterval(fetchBuildHistory, 2000);
