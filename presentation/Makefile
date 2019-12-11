pics/%.pdf: pics/%.svg
	rsvg-convert -f pdf $< > $@

all: slides.pdf

.PHONY: FORCE
slides.pdf: FORCE
	latexmk -pdfxe --synctex=1 -output-directory=latex.out -interaction=nonstopmode -file-line-error slides.tex | sed -re '0,/CONTENT START NOW/d'
	pplatex -q -i latex.out/slides.log
	@grep -v "INFO - " latex.out/slides.blg || true
	cp latex.out/slides.pdf .
	cp latex.out/slides.synctex.gz .

scratch.pdf: FORCE
	latexmk -pdfxe --synctex=1 -output-directory=latex.out -interaction=nonstopmode -file-line-error scratch.tex | cat
	pplatex -q -i latex.out/scratch.log
	@grep -v "INFO - " latex.out/scratch.blg 2>/dev/null || true
	cp latex.out/scratch.pdf .
	cp latex.out/scratch.synctex.gz .


