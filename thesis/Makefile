pics/%.pdf: pics/%.svg
	rsvg-convert -f pdf $< > $@

.PHONY: FORCE
thesis.pdf: FORCE
	latexmk -pdfxe --synctex=1 -output-directory=latex.out -interaction=nonstopmode -file-line-error thesis.tex | sed -re '0,/CONTENT START NOW/d'
	pplatex -q -i latex.out/thesis.log
	cp latex.out/thesis.pdf .
	cp latex.out/thesis.synctex.gz .

