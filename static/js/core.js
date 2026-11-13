/**
 * MyPlace - Core Global JavaScript (Lightweight Shell < 1KB)
 * Handles mobile drawer navigation and global interaction essentials
 */
const mobileNav = {
	isOpen: false,
	toggle() {
		this.isOpen = !this.isOpen;
		const drawer = document.querySelector('.mobile-drawer');
		const overlay = document.querySelector('.mobile-drawer-overlay');
		if (drawer && overlay) {
			if (this.isOpen) {
				drawer.classList.add('open');
				overlay.classList.add('open');
				document.body.style.overflow = 'hidden';
			} else {
				drawer.classList.remove('open');
				overlay.classList.remove('open');
				document.body.style.overflow = '';
			}
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

// Global click-outside listener for mobile drawer
document.addEventListener('click', (e) => {
	if (mobileNav.isOpen && !e.target.closest('.mobile-drawer') && !e.target.closest('.mobile-menu-btn')) {
		mobileNav.close();
	}
});
