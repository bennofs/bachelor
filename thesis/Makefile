pics/%.pdf: pics/%.svg
	rsvg-convert -f pdf $< > $@

all: thesis.pdf scratch.pdf

.PHONY: FORCE
thesis.pdf: FORCE
	latexmk -pdfxe --synctex=1 -output-directory=latex.out -interaction=nonstopmode -file-line-error thesis.tex | sed -re '0,/CONTENT START NOW/d'
	pplatex -q -i latex.out/thesis.log
	@grep -v "INFO - " latex.out/thesis.blg || true
	cp latex.out/thesis.pdf .
	cp latex.out/thesis.synctex.gz .

scratch.pdf: FORCE
	latexmk -pdfxe --synctex=1 -output-directory=latex.out -interaction=nonstopmode -file-line-error scratch.tex | cat
	pplatex -q -i latex.out/scratch.log
	@grep -v "INFO - " latex.out/scratch.blg 2>/dev/null || true
	cp latex.out/scratch.pdf .
	cp latex.out/scratch.synctex.gz .


