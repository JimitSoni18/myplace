/**
 * MyPlace - Projects Catalog Controller
 * Loaded ONLY on /projects
 */

// Catalog Filter Drawer Controller (Mobile Bottom-Sheet)
function toggleFilterDrawer() {
	const drawer = document.getElementById('filter-drawer');
	const overlay = document.getElementById('filter-drawer-overlay');
	if (drawer && overlay) {
		drawer.classList.toggle('open');
		overlay.classList.toggle('open');
		document.body.style.overflow = drawer.classList.contains('open') ? 'hidden' : '';
	}
}

function closeFilterDrawer() {
	const drawer = document.getElementById('filter-drawer');
	const overlay = document.getElementById('filter-drawer-overlay');
	if (drawer && overlay) {
		drawer.classList.remove('open');
		overlay.classList.remove('open');
		document.body.style.overflow = '';
	}
}

// Client-Side Projects Sorter
function sortProjectsGrid(sortValue) {
	const grid = document.querySelector('.cards-grid');
	if (!grid) return;
	const cards = Array.from(grid.querySelectorAll('.entity-card'));

	cards.sort((a, b) => {
		const priceTextA = a.querySelector('.card-price-val')?.textContent || '';
		const priceTextB = b.querySelector('.card-price-val')?.textContent || '';
		const nameA = (a.getAttribute('data-name') || '').toLowerCase();
		const nameB = (b.getAttribute('data-name') || '').toLowerCase();

		if (sortValue === 'price-asc') {
			return priceTextA.localeCompare(priceTextB, undefined, { numeric: true });
		} else if (sortValue === 'price-desc') {
			return priceTextB.localeCompare(priceTextA, undefined, { numeric: true });
		} else if (sortValue === 'name-asc') {
			return nameA.localeCompare(nameB);
		}
		return 0;
	});

	cards.forEach(card => grid.appendChild(card));
}
