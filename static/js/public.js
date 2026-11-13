/**
 * MyPlace - Legacy / Fallback Public JS Bundle
 */
// Mobile navigation fallback
const mobileNav = {
	isOpen: false,
	toggle() {
		this.isOpen = !this.isOpen;
		const drawer = document.querySelector('.mobile-drawer');
		const overlay = document.querySelector('.mobile-drawer-overlay');
		if (drawer && overlay) {
			drawer.classList.toggle('open', this.isOpen);
			overlay.classList.toggle('open', this.isOpen);
			document.body.style.overflow = this.isOpen ? 'hidden' : '';
		}
	},
	close() {
		this.isOpen = false;
		const drawer = document.querySelector('.mobile-drawer');
		const overlay = document.querySelector('.mobile-drawer-overlay');
		if (drawer && overlay) {
			drawer.classList.remove('open');
			overlay.classList.remove('open');
			document.body.style.overflow = '';
		}
	}
};
