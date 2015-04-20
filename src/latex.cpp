#include <iostream>
#include <fstream>
#include <vector>
#include "latex.hpp"

using namespace std;

void test()
{
    double tab0[15] = {2.0,5.6,7.4,-8.9,5,2,3,6,4,6,-7,8,2,4,7};
    double tab1[3] = {2.4,-5.4,8.9};
    double tab2[5] = {-1.0,5.5,6.4,-7.2,2.1};
    vector<double> A (tab0,tab0+sizeof(tab0)/sizeof(double)) ;
    vector<double> B (tab1,tab1+sizeof(tab1)/sizeof(double)) ;
    vector<double> C (tab2,tab2+sizeof(tab2)/sizeof(double)) ;

    init_tex();
    print_systeme(A,B,C);
    end_tex();
}

void init_tex(){
    ofstream latex;
    latex.open("result.tex");

    // intro of the tex file
    latex << "\\documentclass[10pt]{article}"<< endl;
    latex << "\\usepackage[latin1]{inputenc}"<< endl;
    latex << "\\usepackage[T1]{fontenc}"<< endl;
    latex << "\\usepackage[french]{babel}"<< endl;
    latex << "\\usepackage{setspace}"<< endl;
    latex << "\\usepackage{lmodern}"<< endl;
    latex << "\\usepackage{soul}"<< endl;
    latex << "\\usepackage{ulem}"<< endl;
    latex << "\\usepackage{enumerate}"<< endl;
    latex << "\\usepackage{amsmath,amsfonts, amssymb}"<< endl;
    latex << "\\usepackage{mathrsfs}"<< endl;
    latex << "\\usepackage{amsthm}"<< endl;
    latex << "\\usepackage{float}"<< endl;
    latex << "\\usepackage{array}"<< endl;
    latex << "\\usepackage{mathabx}"<< endl;
    latex << "\\usepackage{stmaryrd}"<< endl;
    latex << endl;
    latex << "\\begin{document}"<< endl;

    latex.close();
}

//write in the tex file the system ( A is the matrix, B the constraints, C the optimisation function)
void print_systeme(vector<double> A, vector<double> B, vector<double> C){
    unsigned i,j;
    ofstream latex;
    latex.open("result.tex", ios::app);



    //first line (maximize)
    latex << "Maximize $ ";
    if(C[0]!=0){
        if(C[0]!=1 && C[0]!=-1) latex << C[0];
        if(C[0]==-1) latex << "-";
        latex << "x_{" << 0 << "}";
    }
    for(i=1;i<C.size();i++){
        if(C[i]!=0){
            if (C[i]>0){
                latex << "+";
            }
            latex << C[i] << "x_{" << i << "}";
        }
    }

    //begin of the array
    latex << " $ such that : $ \\\\" << endl;
    latex << "\\left\\{" << endl;
    latex << "\\begin{array}{";
    for(i=0;i<3*C.size();i++) latex << "c";
    latex << "}" << endl;

    //constraints
    for(i=0;i<B.size();i++){
        if(A[i*C.size()]<0){
            latex << "- & ";
            if(A[i*C.size()]!=-1) latex << -A[i*C.size()];
        }
        else{
            latex <<"& ";
            if(A[i*C.size()]>0 && A[i*C.size()] != 1) latex << A[i*C.size()];
        }
        latex << " x_{" << 0 << "} & ";
        for(j=1;j<C.size();j++){
            if(C[j]==0) latex << "& & ";
            else{
                if(A[i*C.size()+j]<0){
                    latex << "- & " ;
                    if (A[i*C.size()+j] != -1) latex << - A[i*C.size()+j];
                }
                else{
                    latex <<"+ & ";
                    if (A[i*C.size()+j] != 1) latex << A[i*C.size()+j];
                }
                latex << " x_{" << j << "} & ";
            }
        }
        latex << "\\\\" << endl;
    }

    //end of the array
    latex << "\\end{array}" << endl;
    latex << "\\right." << endl;
    latex << "$" << endl;


}

void end_tex(){
    ofstream latex;
    latex.open("result.tex", ios::app);

    //end of the document
    latex << "\\end{document}"<< endl;
    latex.close();
}

