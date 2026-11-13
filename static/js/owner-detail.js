/**
 * MyPlace - Developer Showcase Controller
 * Loaded ONLY on /owners/{slug}
 */

function filterOwnerProjects(statusFilter, btn) {
	document.querySelectorAll('[data-owner-filter]').forEach(b => b.classList.remove('active'));
	if (btn) btn.classList.add('active');

	const cards = document.querySelectorAll('#owner-portfolio-grid .entity-card');
	cards.forEach(card => {
		const pos = card.getAttribute('data-possession') || '';
		if (statusFilter === 'all') {
			card.style.display = 'flex';
		} else if (statusFilter === 'ready') {
			card.style.display = pos.includes('ready') ? 'flex' : 'none';
		} else if (statusFilter === 'ongoing') {
			card.style.display = !pos.includes('ready') ? 'flex' : 'none';
		}
	});
}
