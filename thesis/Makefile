.PHONY: FORCE
thesis.pdf: FORCE
	latexmk -pdfxe --synctex=1 -output-directory=latex.out -interaction=nonstopmode -file-line-error thesis.tex | pplatex -i -
	cp latex.out/thesis.synctex.gz .
	cp latex.out/thesis.pdf .

