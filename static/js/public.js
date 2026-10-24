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

// Properties Directory Filter Helpers
function setBudgetPreset(min, max) {
	const minInput = document.getElementById('min_price_input');
	const maxInput = document.getElementById('max_price_input');
	if (minInput) minInput.value = min !== null && min !== undefined ? min : '';
	if (maxInput) maxInput.value = max !== null && max !== undefined ? max : '';
	const form = document.getElementById('property-filters-form');
	if (form) form.submit();
}

function clearBudgetInputs() {
	const minInput = document.getElementById('min_price_input');
	const maxInput = document.getElementById('max_price_input');
	if (minInput) minInput.value = '';
	if (maxInput) maxInput.value = '';
	document.querySelectorAll('.preset-chip.active').forEach(el => el.classList.remove('active'));
}

function clearTypeCheckboxes() {
	document.querySelectorAll('.types-dropdown-menu input[type="checkbox"]').forEach(cb => {
		cb.checked = false;
	});
}

// Global click-outside listener to close filter dropdowns
document.addEventListener('click', (e) => {
	if (!e.target.closest('.filter-dropdown')) {
		document.querySelectorAll('.filter-dropdown[open]').forEach(d => {
			d.removeAttribute('open');
		});
	}
});

// Mutually exclusive dropdown open behavior
document.addEventListener('DOMContentLoaded', () => {
	document.querySelectorAll('.filter-dropdown').forEach(dropdown => {
		dropdown.addEventListener('toggle', () => {
			if (dropdown.open) {
				document.querySelectorAll('.filter-dropdown').forEach(other => {
					if (other !== dropdown && other.open) {
						other.removeAttribute('open');
					}
				});
			}
		});
	});
});

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

