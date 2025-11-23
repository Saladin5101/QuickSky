# Documentation build system for QuickSky

.PHONY: all man info clean install install-man install-info

# Default target
all: man info

# Build man page (already in correct format)
man: sky.1

# Build info documentation from texinfo source
info: sky.info

sky.info: sky.texi
	makeinfo sky.texi

# Clean generated files
clean:
	rm -f sky.info

# Install documentation (requires root)
install: install-man install-info

# Install man page
install-man: sky.1
	install -d $(DESTDIR)/usr/share/man/man1
	install -m 644 sky.1 $(DESTDIR)/usr/share/man/man1/

# Install info documentation
install-info: sky.info
	install -d $(DESTDIR)/usr/share/info
	install -m 644 sky.info $(DESTDIR)/usr/share/info/
	install-info --dir-file=$(DESTDIR)/usr/share/info/dir $(DESTDIR)/usr/share/info/sky.info

# Test man page locally
test-man:
	man ./sky.1

# Test info page locally  
test-info: sky.info
	info ./sky.info