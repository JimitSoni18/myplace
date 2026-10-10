/**
 * MyPlace - Public Progressive Interactivity with Petite-Vue
 */

// Mobile Drawer State
function createMobileNav() {
	return {
		isOpen: false,
		toggle() {
			this.isOpen = !this.isOpen;
			document.body.style.overflow = this.isOpen ? 'hidden' : '';
		},
		close() {
			this.isOpen = false;
			document.body.style.overflow = '';
		}
	};
}

// Interactive Media Carousel & Lightbox
function createGallery(itemsCount = 1) {
	return {
		currentIndex: 0,
		total: itemsCount,
		lightboxOpen: false,
		next() {
			if (this.total > 0) {
				this.currentIndex = (this.currentIndex + 1) % this.total;
				this.scrollThumbIntoView();
			}
		},
		prev() {
			if (this.total > 0) {
				this.currentIndex = (this.currentIndex - 1 + this.total) % this.total;
				this.scrollThumbIntoView();
			}
		},
		selectIndex(idx) {
			this.currentIndex = idx;
			this.scrollThumbIntoView();
		},
		openLightbox() {
			this.lightboxOpen = true;
			document.body.style.overflow = 'hidden';
		},
		closeLightbox() {
			this.lightboxOpen = false;
			document.body.style.overflow = '';
		},
		scrollThumbIntoView() {
			const activeThumb = document.querySelector(`.carousel-thumb-item[data-index="${this.currentIndex}"]`);
			if (activeThumb) {
				activeThumb.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
			}
		},
		mounted() {
			window.addEventListener('keydown', (e) => {
				if (this.lightboxOpen) {
					if (e.key === 'Escape') this.closeLightbox();
					if (e.key === 'ArrowRight') this.next();
					if (e.key === 'ArrowLeft') this.prev();
				}
			});
		}
	};
}

// Interactive EMI / Mortgage Calculator
function createEmiCalculator(initialPrincipal = 7500000) {
	return {
		principal: initialPrincipal,
		downPaymentPercent: 20,
		interestRate: 8.5,
		tenureYears: 20,
		get loanAmount() {
			return Math.round(this.principal * (1 - this.downPaymentPercent / 100));
		},
		get monthlyRate() {
			return this.interestRate / 12 / 100;
		},
		get totalMonths() {
			return this.tenureYears * 12;
		},
		get emi() {
			const P = this.loanAmount;
			const r = this.monthlyRate;
			const n = this.totalMonths;
			if (P <= 0 || r <= 0 || n <= 0) return 0;
			const emiVal = (P * r * Math.pow(1 + r, n)) / (Math.pow(1 + r, n) - 1);
			return Math.round(emiVal);
		},
		get formattedEmi() {
			return '₹ ' + this.emi.toLocaleString('en-IN');
		},
		get formattedLoanAmount() {
			if (this.loanAmount >= 10000000) {
				return '₹ ' + (this.loanAmount / 10000000).toFixed(2) + ' Cr';
			} else if (this.loanAmount >= 100000) {
				return '₹ ' + (this.loanAmount / 100000).toFixed(2) + ' L';
			}
			return '₹ ' + this.loanAmount.toLocaleString('en-IN');
		}
	};
}

// Inventory Filter for Project Detail
function createInventoryFilter() {
	return {
		activeType: 'ALL',
		setType(type) {
			this.activeType = type;
		},
		matches(propType) {
			if (this.activeType === 'ALL') return true;
			return propType.toUpperCase() === this.activeType.toUpperCase();
		}
	};
}

// Initialize Petite-Vue if available
document.addEventListener('DOMContentLoaded', () => {
	if (window.PetiteVue) {
		window.PetiteVue.createApp({
			mobileNav: createMobileNav(),
			createGallery,
			createEmiCalculator,
			createInventoryFilter
		}).mount();
	}
});
